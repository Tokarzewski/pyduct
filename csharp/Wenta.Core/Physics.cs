using System;

namespace Wenta
{
    public static class Friction
    {
        public const double LaminarReLimit = 2300.0;

        public static double Reynolds(double velocity, double hydraulicDiameter,
                                      double kinematicViscosity)
        {
            return velocity * hydraulicDiameter / kinematicViscosity;
        }

        public static double RelativeRoughness(double absoluteRoughness,
                                               double hydraulicDiameter)
        {
            return absoluteRoughness / hydraulicDiameter;
        }

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

    public static class Losses
    {
        public static double StraightPressureDrop(double frictionFactor, double length,
            double hydraulicDiameter, double velocity, double density)
        {
            return frictionFactor * (length / hydraulicDiameter) * density * velocity * velocity / 2.0;
        }

        public static double LocalPressureDrop(double zeta, double velocity, double density)
        {
            return zeta * density * velocity * velocity / 2.0;
        }
    }

    public static class Flex
    {
        public static double StretchCorrectionFactor(double diameter, double stretchPercentage)
        {
            return 0.557 * (100.0 - stretchPercentage) * Math.Exp(-4.93 * diameter) + 1.0;
        }
    }
}
