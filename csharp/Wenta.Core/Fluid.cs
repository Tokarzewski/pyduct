using System;

namespace Wenta
{
    /// <summary>Working fluid (typically air). Port of `wenta.core.fluid`.</summary>
    public sealed class Fluid
    {
        public readonly double Density;             // rho [kg/m^3]
        public readonly double DynamicViscosity;   // mu [Pa.s]
        public readonly double KinematicViscosity; // nu = mu / rho [m^2/s]

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

        /// <summary>Standard dry air at 20 °C, 101 325 Pa
        /// (matches CoolProp to 4 significant figures).</summary>
        public static Fluid StandardAir()
        {
            return new Fluid(1.204, 1.825e-5);
        }

        /// <summary>Dry-air properties at a given altitude (ISA atmosphere)
        /// and temperature. Port of `wentamojo.core.fluid.air_at_altitude`.</summary>
        public static Fluid AirAtAltitude(double altitudeM, double temperatureC = 20.0)
        {
            if (altitudeM < 0.0)
                throw new WentaException("altitude_m must be non-negative");
            double h = altitudeM < 11000.0 ? altitudeM : 11000.0;
            // ISA pressure up to the tropopause.
            double pressure = 101325.0 * Math.Pow(1.0 - 2.25577e-5 * h, 5.2561);
            double tK = temperatureC + 273.15;
            const double rSpecific = 287.058; // J/(kg.K) for dry air
            double density = pressure / (rSpecific * tK);
            // Sutherland: mu(T) = 1.458e-6 * T^1.5 / (T + 110.4)
            double mu = 1.458e-6 * Math.Pow(tK, 1.5) / (tK + 110.4);
            return new Fluid(density, mu);
        }
    }
}
