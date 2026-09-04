using System;

namespace Wenta
{
    /// <summary>Round-elbow loss coefficient from the Hendiger/Ziętek/
    /// Chludzińska table, looked up by R/D and bend angle.
    /// Python uses scipy's RectBivariateSpline (interpolating cubic);
    /// this port uses separable not-a-knot bicubic interpolation, which
    /// agrees with scipy on this 6×10 table (parity tolerance 1e-4).</summary>
    public sealed class ElbowRound
    {
        private static readonly double[] RdGrid = { 0.50, 0.75, 1.00, 1.50, 2.00, 2.50 };
        private static readonly double[] AngleGrid = { 20, 30, 45, 60, 75, 90, 110, 130, 150, 180 };
        private static readonly double[,] ZetaTable =
        {
            {0.22, 0.32, 0.43, 0.55, 0.64, 0.71, 0.80, 0.85, 0.91, 0.99},
            {0.10, 0.15, 0.20, 0.26, 0.30, 0.33, 0.37, 0.40, 0.42, 0.46},
            {0.07, 0.10, 0.13, 0.17, 0.20, 0.22, 0.25, 0.26, 0.28, 0.31},
            {0.05, 0.07, 0.09, 0.12, 0.14, 0.15, 0.17, 0.18, 0.19, 0.21},
            {0.04, 0.06, 0.08, 0.10, 0.12, 0.13, 0.15, 0.16, 0.17, 0.18},
            {0.04, 0.05, 0.07, 0.09, 0.11, 0.12, 0.14, 0.14, 0.15, 0.17},
        };

        public readonly double BendRadius; // R [m]
        public readonly double Diameter;    // D [m]
        public readonly double Angle;       // [deg]

        public ElbowRound(double bendRadius, double diameter, double angle)
        {
            if (diameter <= 0.0)
                throw new WentaException("diameter must be positive, got " + diameter);
            if (bendRadius <= 0.0)
                throw new WentaException("bend_radius must be positive, got " + bendRadius);
            double rd = bendRadius / diameter;
            if (rd < RdGrid[0] || rd > RdGrid[RdGrid.Length - 1])
                throw new WentaException("R/D = " + rd.ToString("0.###")
                    + " is outside the tabulated range ["
                    + RdGrid[0] + ", " + RdGrid[RdGrid.Length - 1] + "]");
            if (angle < AngleGrid[0] || angle > AngleGrid[AngleGrid.Length - 1])
                throw new WentaException("angle = " + angle
                    + " is outside the tabulated range ["
                    + AngleGrid[0] + ", " + AngleGrid[AngleGrid.Length - 1] + "]");
            BendRadius = bendRadius;
            Diameter = diameter;
            Angle = angle;
        }

        public double Zeta()
        {
            double rd = BendRadius / Diameter;
            // Separable bicubic: cubic-spline the angle axis for each R/D row,
            // then cubic-spline the R/D axis through the interpolated values.
            double[] byRd = new double[RdGrid.Length];
            for (int i = 0; i < RdGrid.Length; i++)
                byRd[i] = Spline.Interpolate1D(AngleGrid, GetRow(i), Angle);
            return Spline.Interpolate1D(RdGrid, byRd, rd);
        }

        private double[] GetRow(int i)
        {
            double[] r = new double[AngleGrid.Length];
            for (int j = 0; j < AngleGrid.Length; j++)
                r[j] = ZetaTable[i, j];
            return r;
        }
    }

    /// <summary>Not-a-knot cubic-spline interpolation (single axis).
    /// Unknowns are the second derivatives M[0..n-1] solved from the
    /// n-2 interior curvature equations plus two third-derivative
    /// continuity (not-a-knot) end conditions.</summary>
    internal static class Spline
    {
        public static double Interpolate1D(double[] xs, double[] ys, double x)
        {
            int n = xs.Length;
            if (n == 1) return ys[0];
            if (n == 2)
                return ys[0] + (ys[1] - ys[0]) * (x - xs[0]) / (xs[1] - xs[0]);
            if (x <= xs[0]) x = xs[0];
            if (x >= xs[n - 1]) x = xs[n - 1];

            double[] h = new double[n - 1];
            for (int i = 0; i < n - 1; i++)
                h[i] = xs[i + 1] - xs[i];

            // Build the n x n system  A * M = rhs  (banded, small).
            double[,] a = new double[n, n];
            double[] rhs = new double[n];

            for (int i = 1; i <= n - 2; i++)
            {
                a[i, i - 1] = h[i - 1];
                a[i, i] = 2.0 * (h[i - 1] + h[i]);
                a[i, i + 1] = h[i];
                rhs[i] = 6.0 * ((ys[i + 1] - ys[i]) / h[i] - (ys[i] - ys[i - 1]) / h[i - 1]);
            }
            // not-a-knot left: 3rd-derivative continuity at x1
            a[0, 0] = -h[1];
            a[0, 1] = h[0] + h[1];
            a[0, 2] = -h[0];
            rhs[0] = 0.0;
            // not-a-knot right: 3rd-derivative continuity at x(n-2)
            a[n - 1, n - 1] = -h[n - 3];
            a[n - 1, n - 2] = h[n - 2] + h[n - 3];
            a[n - 1, n - 3] = -h[n - 2];
            rhs[n - 1] = 0.0;

            // Gaussian elimination with partial pivoting (n <= 10 here).
            double[] m = Solve(a, rhs);

            // locate segment and evaluate
            int k = 0;
            while (k < n - 2 && x > xs[k + 1]) k++;
            double hh = h[k];
            double aa = xs[k + 1] - x;
            double bb = x - xs[k];
            return (aa * aa * aa / (6.0 * hh)) * m[k]
                 + (bb * bb * bb / (6.0 * hh)) * m[k + 1]
                 + (aa / hh) * (ys[k] - hh * hh * m[k] / 6.0)
                 + (bb / hh) * (ys[k + 1] - hh * hh * m[k + 1] / 6.0);
        }

        private static double[] Solve(double[,] a, double[] b)
        {
            int n = b.Length;
            for (int col = 0; col < n; col++)
            {
                // pivot
                int piv = col;
                double best = Math.Abs(a[col, col]);
                for (int r = col + 1; r < n; r++)
                {
                    double v = Math.Abs(a[r, col]);
                    if (v > best) { best = v; piv = r; }
                }
                if (piv != col)
                {
                    for (int c = 0; c < n; c++)
                    {
                        double t = a[col, c]; a[col, c] = a[piv, c]; a[piv, c] = t;
                    }
                    double tb = b[col]; b[col] = b[piv]; b[piv] = tb;
                }
                // eliminate below
                for (int r = col + 1; r < n; r++)
                {
                    double w = a[r, col] / a[col, col];
                    if (w == 0.0) continue;
                    for (int c = col; c < n; c++)
                        a[r, c] -= w * a[col, c];
                    b[r] -= w * b[col];
                }
            }
            double[] x = new double[n];
            for (int i = n - 1; i >= 0; i--)
            {
                double s = b[i];
                for (int c = i + 1; c < n; c++)
                    s -= a[i, c] * x[c];
                x[i] = s / a[i, i];
            }
            return x;
        }
    }
}
