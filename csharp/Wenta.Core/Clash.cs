using System;
using System.Collections.Generic;
using System.Text;

namespace Wenta
{
    public class Clash
    {
        public readonly string A;
        public readonly string B;
        public readonly double DistanceM;

        public Clash(string a, string b, double distanceM)
        {
            A = a;
            B = b;
            DistanceM = distanceM;
        }
    }

    public static class ClashDetection
    {
        private const double Epsilon = 2.2204460492503131e-16;

        private static double PointSegment(Point2 p, Point2 a, Point2 b)
        {
            double abx = b.X - a.X;
            double aby = b.Y - a.Y;
            double len2 = abx * abx + aby * aby;
            if (len2 == 0.0)
                return p.DistanceTo(a);

            double apx = p.X - a.X;
            double apy = p.Y - a.Y;
            double t = ReCorrections.Clamp((apx * abx + apy * aby) / len2, 0.0, 1.0);
            Point2 closest = new Point2(a.X + t * abx, a.Y + t * aby);
            return p.DistanceTo(closest);
        }

        private static double EndpointMin(Point2 p0, Point2 p1, Point2 q0, Point2 q1)
        {
            double best = double.PositiveInfinity;
            best = Math.Min(best, PointSegment(p0, q0, q1));
            best = Math.Min(best, PointSegment(p1, q0, q1));
            best = Math.Min(best, PointSegment(q0, p0, p1));
            best = Math.Min(best, PointSegment(q1, p0, p1));
            return best;
        }

        internal static double SegmentDistance(Point2 p0, Point2 p1, Point2 q0, Point2 q1)
        {
            double ux = p1.X - p0.X, uy = p1.Y - p0.Y;
            double vx = q1.X - q0.X, vy = q1.Y - q0.Y;
            double wx = p0.X - q0.X, wy = p0.Y - q0.Y;

            double a = ux * ux + uy * uy;
            double b = ux * vx + uy * vy;
            double c = vx * vx + vy * vy;
            double d = ux * wx + uy * wy;
            double e = vx * wx + vy * wy;

            double denom = a * c - b * b;

            if (denom <= Epsilon)
                return EndpointMin(p0, p1, q0, q1);

            double sC = (b * e - c * d) / denom;
            double tC = (a * e - b * d) / denom;
            if (sC >= 0.0 && sC <= 1.0 && tC >= 0.0 && tC <= 1.0)
            {
                Point2 p = new Point2(p0.X + sC * ux, p0.Y + sC * uy);
                Point2 q = new Point2(q0.X + tC * vx, q0.Y + tC * vy);
                return p.DistanceTo(q);
            }
            return EndpointMin(p0, p1, q0, q1);
        }

        public static List<Clash> FindClashes(IList<DuctSegment> segments, double clearanceM)
        {
            if (clearanceM < 0.0)
                throw new WentaException("clearance_m must be >= 0");

            List<Clash> clashes = new List<Clash>();
            for (int i = 0; i < segments.Count; i++)
            {
                DuctSegment a = segments[i];
                for (int j = i + 1; j < segments.Count; j++)
                {
                    DuctSegment b = segments[j];
                    double dist = SegmentDistance(a.Start, a.End, b.Start, b.End);
                    double threshold = (a.Diameter + b.Diameter) / 2.0 + clearanceM;
                    if (dist < threshold)
                    {
                        bool aFirst = string.CompareOrdinal(a.ComponentId, b.ComponentId) <= 0;
                        clashes.Add(new Clash(
                            aFirst ? a.ComponentId : b.ComponentId,
                            aFirst ? b.ComponentId : a.ComponentId,
                            dist));
                    }
                }
            }
            return clashes;
        }

        public static int ClashCount(IList<Clash> clashes)
        {
            return clashes.Count;
        }

        public static string ClashesAsCsv(IList<Clash> clashes)
        {
            StringBuilder sb = new StringBuilder("a,b,distance_m\n");
            for (int i = 0; i < clashes.Count; i++)
            {
                Clash c = clashes[i];
                sb.Append(c.A).Append(',').Append(c.B).Append(',')
                  .Append(RoomBalanceSet.FmtNum(c.DistanceM)).Append('\n');
            }
            return sb.ToString();
        }
    }
}
