using System;

namespace Wenta
{
    public sealed class Fluid
    {
        public readonly double Density;
        public readonly double DynamicViscosity;
        public readonly double KinematicViscosity;

        public Fluid(double density, double dynamicViscosity)
        {
            if (density <= 0.0)
                throw new WentaException("density must be positive, got " + density);
            if (dynamicViscosity <= 0.0)
                throw new WentaException(
                    "dynamic_viscosity must be positive, got " + dynamicViscosity);
            Density = density;
            DynamicViscosity = dynamicViscosity;
            KinematicViscosity = dynamicViscosity / density;
        }

        public static Fluid StandardAir()
        {
            return new Fluid(1.204, 1.825e-5);
        }

        public static Fluid AirAtAltitude(double altitudeM, double temperatureC = 20.0)
        {
            if (altitudeM < 0.0)
                throw new WentaException("altitude_m must be non-negative");
            double h = altitudeM < 11000.0 ? altitudeM : 11000.0;
            double pressure = 101325.0 * Math.Pow(1.0 - 2.25577e-5 * h, 5.2561);
            double tK = temperatureC + 273.15;
            const double rSpecific = 287.058;
            double density = pressure / (rSpecific * tK);
            double mu = 1.458e-6 * Math.Pow(tK, 1.5) / (tK + 110.4);
            return new Fluid(density, mu);
        }
    }
}
