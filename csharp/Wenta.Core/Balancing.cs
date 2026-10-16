using System;

namespace Wenta
{
    public static class Balancing
    {
        public static double RequiredZeta(double requiredDpPa, double velocity, double density)
        {
            double dynamic = density * velocity * velocity * 0.5;
            if (dynamic <= 0.0) return 0.0;
            return requiredDpPa / dynamic;
        }

        public static double BalancingZeta(double totalReqPa, double branchAvailablePa,
            double velocity, double density)
        {
            double delta = totalReqPa - branchAvailablePa;
            if (delta <= 0.0) return 0.0;
            return RequiredZeta(delta, velocity, density);
        }

        public static double DamperOpenPercentage(double zeta)
        {
            if (zeta <= 0.1) return 100.0;
            double root = Math.Sqrt((zeta - 0.1) / 10.0);
            double open = 100.0 * (1.0 - root);
            return Math.Max(0.0, Math.Min(100.0, open));
        }
    }
}