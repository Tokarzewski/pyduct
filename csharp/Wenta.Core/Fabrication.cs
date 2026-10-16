using System;
using System.Collections.Generic;

namespace Wenta
{
    public static class Fabrication
    {
        public const double SteelDensityKgM3 = 7850.0;

        public static double DuctSurfaceAreaM2(CrossSection crossSection, double lengthM)
        {
            if (lengthM < 0.0)
                throw new WentaException("length must be non-negative");
            return crossSection.Perimeter * lengthM;
        }

        public static double DuctWeightKg(double surfaceAreaM2, double? gaugeMm, double? densityKgM3)
        {
            if (surfaceAreaM2 < 0.0)
                throw new WentaException("surface area must be non-negative");

            if (gaugeMm == null)
                throw new WentaException("gauge (steel thickness) is required to compute weight");
            double gauge = gaugeMm.Value;
            if (gauge <= 0.0)
                throw new WentaException("gauge must be positive");
            double gaugeM = gauge / 1000.0;

            double density = densityKgM3 ?? SteelDensityKgM3;
            if (density <= 0.0)
                throw new WentaException("density must be positive");

            return surfaceAreaM2 * gaugeM * density;
        }

        public sealed class FabricationBreakout
        {
            public readonly double StraightM;

            public readonly double FittingsM;

            public FabricationBreakout(double straight, double fittings)
            {
                if (straight < 0.0 || fittings < 0.0)
                    throw new WentaException("straight and fittings lengths must be non-negative");
                StraightM = straight;
                FittingsM = fittings;
            }

            public double TotalM()
            {
                return StraightM + FittingsM;
            }
        }

        public static List<KeyValuePair<string, double>> CuttingSchedule(
            IEnumerable<KeyValuePair<string, double>> ducts)
        {
            var totals = new Dictionary<string, double>();
            foreach (KeyValuePair<string, double> duct in ducts)
            {
                double existing;
                totals.TryGetValue(duct.Key, out existing);
                totals[duct.Key] = existing + duct.Value;
            }

            var keys = new List<string>(totals.Keys);
            keys.Sort(string.CompareOrdinal);

            var result = new List<KeyValuePair<string, double>>(keys.Count);
            foreach (string k in keys)
                result.Add(new KeyValuePair<string, double>(k, totals[k]));
            return result;
        }
    }
}
