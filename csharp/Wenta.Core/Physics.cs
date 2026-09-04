using System;

namespace Wenta
{
    /// <summary>Friction-related correlations for duct flow.
    /// Port of `wentamojo.physics.friction` (Swamee–Jain + Colebrook–White).</summary>
    public static class Friction
    {
        public const double LaminarReLimit = 2300.0;

        /// <summary>Reynolds number Re = v · D_h / nu.</summary>
        public static double Reynolds(double velocity, double hydraulicDiameter,
                                      double kinematicViscosity)
        {
            return velocity * hydraulicDiameter / kinematicViscosity;
        }

        /// <summary>Relative roughness epsilon / D_h.</summary>
        public static double RelativeRoughness(double absoluteRoughness,
                                               double hydraulicDiameter)
        {
            return absoluteRoughness / hydraulicDiameter;
        }

        /// <summary>Darcy friction factor (Swamee–Jain explicit approximation).
        /// Laminar fallback 64/Re for Re &lt; 2300.</summary>
        public static double FrictionFactor(double reynoldsNumber, double relRoughness)
        {
            if (reynoldsNumber < LaminarReLimit)
                return 64.0 / reynoldsNumber;
            double arg = 0.234 * Math.Pow(relRoughness, 1.1007)
                       - 60.525 / Math.Pow(reynoldsNumber, 1.1105)
                       + 56.291 / Math.Pow(reynoldsNumber, 1.0712);
            double l = Math.Log(arg);
            return 1.613 / (l * l);
        }

        /// <summary>Implicit Colebrook–White friction factor via fixed-point
        /// iteration seeded from Swamee–Jain (tol 1e-12, max 100 iterations).</summary>
        public static double FrictionFactorColebrook(double reynoldsNumber,
                                                     double relRoughness,
                                                     double tol = 1e-12,
                                                     int maxIter = 100)
        {
            if (reynoldsNumber < LaminarReLimit)
                return 64.0 / reynoldsNumber;
            double f = FrictionFactor(reynoldsNumber, relRoughness);
            for (int i = 0; i < maxIter; i++)
            {
                double rhs = -2.0 * Math.Log10(
                    relRoughness / 3.71 + 2.51 / (reynoldsNumber * Math.Sqrt(f)));
                double fNew = 1.0 / (rhs * rhs);
                double diff = fNew >= f ? fNew - f : f - fNew;
                if (diff < tol)
                    return fNew;
                f = fNew;
            }
            return f;
        }
    }

    /// <summary>Pressure-loss primitives. Port of `wentamojo.physics.losses`.</summary>
    public static class Losses
    {
        /// <summary>Darcy–Weisbach: dp = f · (L / D_h) · rho · v² / 2 [Pa].</summary>
        public static double StraightPressureDrop(double frictionFactor, double length,
            double hydraulicDiameter, double velocity, double density)
        {
            return frictionFactor * (length / hydraulicDiameter) * density * velocity * velocity / 2.0;
        }

        /// <summary>Local (minor) loss: dp = zeta · rho · v² / 2 [Pa].</summary>
        public static double LocalPressureDrop(double zeta, double velocity, double density)
        {
            return zeta * density * velocity * velocity / 2.0;
        }
    }

    /// <summary>Flex-duct correction. Port of `wentamojo.physics.flex`.
    /// Curve fit (R² = 0.995) from ASHRAE Fundamentals.</summary>
    public static class Flex
    {
        public static double StretchCorrectionFactor(double diameter, double stretchPercentage)
        {
            return 0.557 * (100.0 - stretchPercentage) * Math.Exp(-4.93 * diameter) + 1.0;
        }
    }
}
