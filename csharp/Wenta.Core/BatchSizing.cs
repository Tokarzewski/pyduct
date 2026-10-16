using System;
using System.Collections.Generic;
using System.Globalization;

namespace Wenta
{
    public enum SizingMethod
    {
        Velocity,
        EqualFriction,
        PressureDropBudget,
        NoiseLimit,
        AspectRatio
    }

    public sealed class BatchSizingRequest
    {
        public string Id;
        public double FlowrateM3s;
        public string Shape = Sizing.ShapeRound;
        public SizingMethod Method = SizingMethod.Velocity;
        public double TargetVelocity = 4.0;
        public double TargetPaPerM = 1.0;
        public double LengthM;
        public double BudgetPa;
        public string SpaceType = "office";
        public double AspectRatio = 2.0;
        public double AbsoluteRoughnessM = RigidDuct.DefaultAbsoluteRoughness;
    }

    public sealed class BatchSizingResult
    {
        public string Id;
        public Sizing.SizingResult Result;
        public int? SnappedRoundMm;
        public int[] SnappedRectMm;
        public string Error;
    }

    public static class BatchSizing
    {
        public static readonly string[] Fields =
        {
            "id",
            "shape",
            "diameter_m",
            "width_m",
            "height_m",
            "velocity_ms",
            "dp_pa_per_m",
            "snapped_round_mm",
            "snapped_rect_w_mm",
            "snapped_rect_h_mm",
            "error",
        };

        public static List<BatchSizingResult> Size(IList<BatchSizingRequest> requests,
            Standard standard = Standard.En1505_1506, Fluid fluid = null)
        {
            if (requests == null)
                throw new WentaException("requests is null");
            fluid = fluid ?? Fluid.StandardAir();

            var results = new List<BatchSizingResult>(requests.Count);
            foreach (BatchSizingRequest req in requests)
            {
                var res = new BatchSizingResult { Id = req != null ? req.Id : null };
                try
                {
                    if (req == null)
                        throw new WentaException("request is null");
                    res.Result = SizeOne(req, fluid);
                    Snap(res, standard);
                }
                catch (WentaException ex)
                {
                    res.Result = null;
                    res.SnappedRoundMm = null;
                    res.SnappedRectMm = null;
                    res.Error = ex.Message;
                }
                results.Add(res);
            }
            return results;
        }

        private static Sizing.SizingResult SizeOne(BatchSizingRequest req, Fluid fluid)
        {
            switch (req.Method)
            {
                case SizingMethod.Velocity:
                    return Sizing.VelocityMethod(req.FlowrateM3s, req.Shape, req.TargetVelocity);
                case SizingMethod.EqualFriction:
                    return Sizing.EqualFrictionMethod(req.FlowrateM3s, req.TargetPaPerM,
                        req.Shape, req.AbsoluteRoughnessM, fluid);
                case SizingMethod.PressureDropBudget:
                    return Sizing.PressureDropBudget(req.FlowrateM3s, req.LengthM, req.BudgetPa,
                        req.Shape, req.AbsoluteRoughnessM, fluid);
                case SizingMethod.NoiseLimit:
                    return Sizing.NoiseLimitMethod(req.FlowrateM3s, req.SpaceType, req.Shape,
                        req.AbsoluteRoughnessM, fluid);
                case SizingMethod.AspectRatio:
                    return Sizing.AspectRatioMethod(req.FlowrateM3s, req.TargetVelocity,
                        req.AspectRatio);
                default:
                    throw new WentaException("unknown sizing method " + req.Method);
            }
        }

        private static double ToMm(double metres)
        {
            return Math.Round(metres * 1000.0, 3);
        }

        private static void Snap(BatchSizingResult res, Standard standard)
        {
            Round round = res.Result.Section as Round;
            if (round != null)
            {
                res.SnappedRoundMm = Standards.NearestRoundSizeFor(standard, ToMm(round.Diameter), true);
                return;
            }
            Rectangular rect = res.Result.Section as Rectangular;
            if (rect != null)
                res.SnappedRectMm = SnapRectangular(standard, ToMm(rect.Width), ToMm(rect.Height));
        }

        public static int[] SnapRectangular(Standard standard, double widthMm, double heightMm)
        {
            int[] best = null;
            long bestArea = long.MaxValue;
            foreach (int[] wh in Standards.StandardRectangularSizes(standard))
            {
                if (wh[0] < widthMm || wh[1] < heightMm) continue;
                long area = (long)wh[0] * wh[1];
                if (area < bestArea)
                {
                    bestArea = area;
                    best = wh;
                }
            }
            return best == null ? null : new[] { best[0], best[1] };
        }

        public static string ToCsv(IList<BatchSizingResult> results)
        {
            var lines = new List<string> { string.Join(",", Fields) };
            if (results == null) return lines[0];
            foreach (BatchSizingResult r in results)
            {
                string shape = "", diameter = "", width = "", height = "", velocity = "", dpPerM = "";
                if (r.Result != null)
                {
                    Round round = r.Result.Section as Round;
                    Rectangular rect = r.Result.Section as Rectangular;
                    if (round != null)
                    {
                        shape = Sizing.ShapeRound;
                        diameter = round.Diameter.ToString(CultureInfo.InvariantCulture);
                    }
                    else if (rect != null)
                    {
                        shape = Sizing.ShapeRectangular;
                        width = rect.Width.ToString(CultureInfo.InvariantCulture);
                        height = rect.Height.ToString(CultureInfo.InvariantCulture);
                    }
                    velocity = r.Result.Velocity.ToString(CultureInfo.InvariantCulture);
                    if (r.Result.PressureDropPerMeter != 0.0)
                        dpPerM = r.Result.PressureDropPerMeter.ToString(CultureInfo.InvariantCulture);
                }
                lines.Add(string.Join(",", new[]
                {
                    Quote(r.Id),
                    shape,
                    diameter,
                    width,
                    height,
                    velocity,
                    dpPerM,
                    r.SnappedRoundMm.HasValue
                        ? r.SnappedRoundMm.Value.ToString(CultureInfo.InvariantCulture) : "",
                    r.SnappedRectMm != null
                        ? r.SnappedRectMm[0].ToString(CultureInfo.InvariantCulture) : "",
                    r.SnappedRectMm != null
                        ? r.SnappedRectMm[1].ToString(CultureInfo.InvariantCulture) : "",
                    Quote(r.Error),
                }));
            }
            return string.Join("\n", lines.ToArray());
        }

        private static string Quote(string s)
        {
            if (s == null) return "";
            if (s.IndexOfAny(new[] { ',', '"', '\n', '\r' }) < 0) return s;
            return "\"" + s.Replace("\"", "\"\"") + "\"";
        }
    }
}
