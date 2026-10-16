using System;

namespace Wenta
{
    public struct InsulationMaterial
    {
        public string Name;
        public double Conductivity;

        public InsulationMaterial(string name, double conductivity)
        {
            Name = name;
            Conductivity = conductivity;
        }
    }

    public static class Insulation
    {
        public const double DefaultInnerHtc = 10.0;

        public const double DefaultOuterHtc = 8.0;

        public const double MaxThicknessM = 0.250;

        public static readonly double[] StandardThicknessMm =
        {
            20.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0, 120.0,
        };

        public static readonly InsulationMaterial[] Materials =
        {
            new InsulationMaterial("mineral_wool", 0.035),
            new InsulationMaterial("pe_foam", 0.040),
            new InsulationMaterial("epdm_nbr", 0.038),
            new InsulationMaterial("pir", 0.024),
            new InsulationMaterial("pu_foam", 0.028),
        };

        public static bool MaterialConductivity(string name, out double conductivity)
        {
            foreach (InsulationMaterial m in Materials)
            {
                if (string.Equals(m.Name, name, StringComparison.OrdinalIgnoreCase))
                {
                    conductivity = m.Conductivity;
                    return true;
                }
            }
            conductivity = 0.0;
            return false;
        }

        private static double RIns(double cond, double d, double dO)
        {
            return Math.Log(dO / d) / (2.0 * Math.PI * cond);
        }

        private static double HeatFlow(double airTempC, double ambientTempC, double cond,
            double dM, double tM, double innerHtc, double outerHtc)
        {
            double dO = dM + 2.0 * tM;
            double rIn = 1.0 / (innerHtc * Math.PI * dM);
            double rIns = RIns(cond, dM, dO);
            double rOut = 1.0 / (outerHtc * Math.PI * dO);
            return (airTempC - ambientTempC) / (rIn + rIns + rOut);
        }

        private static double SurfaceTemp(double airTempC, double ambientTempC, double cond,
            double dM, double tM, double innerHtc, double outerHtc)
        {
            double q = HeatFlow(airTempC, ambientTempC, cond, dM, tM, innerHtc, outerHtc);
            double dO = dM + 2.0 * tM;
            double rOut = 1.0 / (outerHtc * Math.PI * dO);
            return ambientTempC + q * rOut;
        }

        public static double RequiredThicknessCondensation(double airTempC, double dewPointC,
            double ambientTempC, double conductivity, double ductOuterDiameterM,
            double innerHtc, double outerHtc)
        {
            Validate(ductOuterDiameterM, conductivity, innerHtc, outerHtc);
            return SmallestThickness(
                delegate (double t)
                {
                    return SurfaceTemp(airTempC, ambientTempC, conductivity, ductOuterDiameterM, t,
                        innerHtc, outerHtc) >= dewPointC;
                },
                "dew point cannot be reached within the maximum considered thickness");
        }

        private static double SmallestThickness(Func<double, bool> ok, string failMessage)
        {
            if (ok(0.0))
                return 0.0;
            const double step = 0.001;
            double t = 0.0;
            while (t <= MaxThicknessM)
            {
                t += step;
                if (ok(t))
                    return t;
            }
            throw new WentaException(failMessage);
        }

        public static double RequiredThicknessHeatLoss(double airTempC, double ambientTempC,
            double targetWPerM, double conductivity, double ductOuterDiameterM,
            double innerHtc, double outerHtc)
        {
            Validate(ductOuterDiameterM, conductivity, innerHtc, outerHtc);
            if (targetWPerM <= 0.0)
                throw new WentaException("target_w_per_m must be positive");
            return SmallestThickness(
                delegate (double t)
                {
                    return Math.Abs(HeatFlow(airTempC, ambientTempC, conductivity,
                        ductOuterDiameterM, t, innerHtc, outerHtc)) <= targetWPerM;
                },
                "heat-loss target cannot be met within the maximum considered thickness");
        }

        public static double HeatLossWithInsulation(double airTempC, double ambientTempC,
            double conductivity, double ductOuterDiameterM, double thicknessM,
            double innerHtc, double outerHtc)
        {
            Validate(ductOuterDiameterM, conductivity, innerHtc, outerHtc);
            if (thicknessM < 0.0)
                throw new WentaException("thickness_m must be non-negative");
            return HeatFlow(airTempC, ambientTempC, conductivity, ductOuterDiameterM,
                thicknessM, innerHtc, outerHtc);
        }

        public static double SelectThickness(double requiredM)
        {
            if (requiredM < 0.0)
                throw new WentaException("required_m must be non-negative");
            double requiredMm = requiredM * 1000.0;
            double max = StandardThicknessMm[StandardThicknessMm.Length - 1];
            if (requiredMm > max)
            {
                throw new WentaException(string.Format(
                    "required thickness {0:F0} mm exceeds the largest standard step {1:F0} mm",
                    requiredMm, max));
            }
            foreach (double s in StandardThicknessMm)
            {
                if (s >= requiredMm)
                    return s / 1000.0;
            }
            throw new WentaException("largest standard step covers the required range");
        }

        private static void Validate(double dM, double cond, double innerHtc, double outerHtc)
        {
            if (dM <= 0.0)
                throw new WentaException("duct_outer_diameter_m must be positive");
            if (cond <= 0.0)
                throw new WentaException("conductivity must be positive");
            if (innerHtc <= 0.0 || outerHtc <= 0.0)
                throw new WentaException("heat-transfer coefficients must be positive");
        }
    }
}
