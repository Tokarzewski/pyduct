using System;
using System.Collections.Generic;

namespace Wenta
{
    /// <summary>Standard EN duct sizes (mm). Port of `wenta.data.standard_sizes`.</summary>
    public static class StandardSizes
    {
        /// <summary>EN 1505:2001 rectangular ducts (width × height, mm).</summary>
        public static readonly int[][] RectangularDuctSizes =
        {
            new[] {100, 200}, new[] {150, 200}, new[] {200, 200}, new[] {100, 250}, new[] {150, 250},
            new[] {200, 250}, new[] {250, 250}, new[] {100, 300}, new[] {150, 300}, new[] {200, 300},
            new[] {250, 300}, new[] {300, 300}, new[] {100, 400}, new[] {150, 400}, new[] {200, 400},
            new[] {250, 400}, new[] {300, 400}, new[] {400, 400}, new[] {150, 500}, new[] {200, 500},
            new[] {250, 500}, new[] {300, 500}, new[] {400, 500}, new[] {500, 500}, new[] {150, 600},
            new[] {200, 600}, new[] {250, 600}, new[] {300, 600}, new[] {400, 600}, new[] {500, 600},
            new[] {600, 600}, new[] {200, 800}, new[] {250, 800}, new[] {300, 800}, new[] {400, 800},
            new[] {500, 800}, new[] {600, 800}, new[] {800, 800}, new[] {250, 1000}, new[] {300, 1000},
            new[] {400, 1000}, new[] {500, 1000}, new[] {600, 1000}, new[] {800, 1000}, new[] {1000, 1000},
            new[] {300, 1200}, new[] {400, 1200}, new[] {500, 1200}, new[] {600, 1200}, new[] {800, 1200},
            new[] {1000, 1200}, new[] {1200, 1200}, new[] {400, 1400}, new[] {500, 1400}, new[] {600, 1400},
            new[] {800, 1400}, new[] {1000, 1400}, new[] {1200, 1400}, new[] {400, 1600}, new[] {500, 1600},
            new[] {600, 1600}, new[] {800, 1600}, new[] {1000, 1600}, new[] {1200, 1600}, new[] {500, 1800},
            new[] {600, 1800}, new[] {800, 1800}, new[] {1000, 1800}, new[] {1200, 1800}, new[] {500, 2000},
            new[] {600, 2000}, new[] {800, 2000}, new[] {1000, 2000}, new[] {1200, 2000},
        };

        /// <summary>EN 1506:2007 round ducts (nominal diameter, mm).</summary>
        public static readonly int[] RoundDuctSizes =
        {
            63, 80, 100, 125, 150, 160, 200, 250, 300, 315, 355, 400,
            450, 500, 560, 630, 710, 800, 900, 1000, 1120, 1250,
        };

        private static Round[] _roundSections;
        private static Rectangular[] _rectSections;

        public static Round[] RoundSections()
        {
            if (_roundSections == null)
            {
                var list = new List<Round>(RoundDuctSizes.Length);
                foreach (int d in RoundDuctSizes)
                    list.Add(new Round(d / 1000.0));
                _roundSections = list.ToArray();
            }
            return _roundSections;
        }

        public static Rectangular[] RectangularSections()
        {
            if (_rectSections == null)
            {
                var list = new List<Rectangular>(RectangularDuctSizes.Length);
                foreach (int[] wh in RectangularDuctSizes)
                    list.Add(new Rectangular(wh[0] / 1000.0, wh[1] / 1000.0));
                _rectSections = list.ToArray();
            }
            return _rectSections;
        }

        /// <summary>Nearest EN 1506 nominal diameter [mm]. With roundUp=true
        /// (default) picks the smallest standard size ≥ diameter_mm; otherwise
        /// the closest standard size in either direction.</summary>
        public static int NearestRoundSize(double diameterMm, bool roundUp = true)
        {
            int[] sizes = RoundDuctSizes;
            int n = sizes.Length;
            int first = sizes[0];
            int last = sizes[n - 1];
            if (diameterMm <= first)
                return first;
            if (diameterMm >= last)
                return last;
            int idx = 0;
            for (int i = 0; i < n; i++)
            {
                if (sizes[i] >= diameterMm)
                {
                    idx = i;
                    break;
                }
            }
            if (roundUp)
                return sizes[idx];
            // closest in either direction
            int prev = sizes[idx - 1];
            return (diameterMm - prev) < (sizes[idx] - diameterMm) ? prev : sizes[idx];
        }
    }
}
