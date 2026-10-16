using System;
using System.Collections.Generic;

namespace Wenta
{
    public static class Sound
    {
        private static readonly double RhoZero = Fluid.StandardAir().Density;

        private const double RegenC = 10.0;

        public static readonly Dictionary<string, double> NoiseLimitsNc =
            new Dictionary<string, double>
        {
            { "studio", 25.0 },
            { "bedroom", 25.0 },
            { "office", 35.0 },
            { "classroom", 35.0 },
            { "retail", 40.0 },
            { "industrial", 60.0 },
        };

        public static double RegeneratedNoiseRound(double velocity, double diameter,
            double? density = null)
        {
            if (velocity <= 0.0)
                throw new WentaException("velocity must be positive, got " + velocity);
            if (diameter <= 0.0)
                throw new WentaException("diameter must be positive, got " + diameter);
            double rho = density ?? RhoZero;
            if (rho <= 0.0)
                throw new WentaException("density must be positive, got " + rho);

            return RegenC + 10.0 * Math.Log10(rho / RhoZero) + 60.0 * Math.Log10(velocity)
                - 20.0 * Math.Log10(diameter);
        }

        public static double DuctPressureLevel(double soundPowerDb, double roomSurfaceArea,
            double absorptionCoefficient)
        {
            if (roomSurfaceArea <= 0.0)
                throw new WentaException(
                    "room_surface_area must be positive, got " + roomSurfaceArea);
            if (!(absorptionCoefficient > 0.0 && absorptionCoefficient < 1.0))
                throw new WentaException(
                    "absorption_coefficient must be in (0, 1), got " + absorptionCoefficient);

            double roomTerm = 4.0 * (1.0 - absorptionCoefficient)
                / (absorptionCoefficient * roomSurfaceArea);
            return soundPowerDb + 10.0 * Math.Log10(roomTerm);
        }

        private static double NcLimit(string spaceType)
        {
            double limit;
            if (!NoiseLimitsNc.TryGetValue(spaceType, out limit))
                throw new WentaException(
                    "unknown space_type '" + spaceType
                    + "'; expected one of studio|bedroom|office|classroom|retail|industrial");
            return limit;
        }

        public static bool NcOk(string spaceType, double levelDb)
        {
            return NcOkTarget(NcLimit(spaceType), levelDb);
        }

        public static bool NcOkTarget(double ncTarget, double levelDb)
        {
            return levelDb <= ncTarget + 1e-9;
        }
    }
}
