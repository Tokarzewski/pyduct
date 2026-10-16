using System;

namespace Wenta
{
    public static class ReCorrections
    {
        public const double ReRef = 50000.0;

        public const double DRefM = 0.200;

        internal static double Clamp(double v, double lo, double hi)
        {
            return Math.Max(lo, Math.Min(hi, v));
        }

        private static void RequirePositive(double value, string name)
        {
            if (value <= 0.0)
                throw new WentaException(name + " must be positive");
        }

        public static double ReCorrection(double reynoldsNumber, double exponent)
        {
            RequirePositive(reynoldsNumber, "reynolds_number");
            return Clamp(Math.Pow(ReRef / reynoldsNumber, exponent), 0.75, 1.5);
        }

        public static double SizeCorrection(double ductDiameterM, double exponent)
        {
            RequirePositive(ductDiameterM, "duct_diameter_m");
            return Clamp(Math.Pow(DRefM / ductDiameterM, exponent), 0.9, 1.3);
        }

        public static double CorrectedZeta(double baseZeta, double velocity, double ductDiameterM,
            double reExponent, double sizeExponent, Fluid fluid = null)
        {
            fluid = fluid ?? Fluid.StandardAir();
            double re = Friction.Reynolds(velocity, ductDiameterM, fluid.KinematicViscosity);
            double factor = ReCorrection(re, reExponent) * SizeCorrection(ductDiameterM, sizeExponent);
            return baseZeta * factor;
        }
    }
}
