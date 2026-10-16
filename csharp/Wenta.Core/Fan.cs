using System;

namespace Wenta
{
    public struct FanPoint
    {
        public readonly double FlowM3s;
        public readonly double StaticPressurePa;

        public FanPoint(double flowM3s, double staticPressurePa)
        {
            FlowM3s = flowM3s;
            StaticPressurePa = staticPressurePa;
        }
    }

    public sealed class FanCurve
    {
        public readonly string Name;
        public readonly FanPoint[] Points;

        public FanCurve(string name, FanPoint[] points)
        {
            if (points == null || points.Length < 2)
                throw new WentaException("fan curve '" + name + "' needs at least 2 points, got "
                    + (points == null ? 0 : points.Length));
            for (int i = 0; i < points.Length - 1; i++)
            {
                FanPoint a = points[i];
                FanPoint b = points[i + 1];
                if (!IsFinite(a.FlowM3s) || !IsFinite(a.StaticPressurePa)
                    || !IsFinite(b.FlowM3s) || !IsFinite(b.StaticPressurePa))
                    throw new WentaException("fan curve '" + name + "' contains a non-finite point");
                if (a.FlowM3s >= b.FlowM3s)
                    throw new WentaException("fan curve '" + name + "' flows must be strictly increasing ("
                        + a.FlowM3s + " m3/s then " + b.FlowM3s + " m3/s)");
            }
            Name = name;
            Points = points;
        }

        public double StaticPressureAt(double flowM3s)
        {
            if (!IsFinite(flowM3s))
                throw new WentaException("fan '" + Name + "': flow " + flowM3s + " m3/s is not finite");
            FanPoint first = Points[0];
            FanPoint last = Points[Points.Length - 1];
            if (flowM3s < first.FlowM3s || flowM3s > last.FlowM3s)
                throw new WentaException("fan '" + Name + "': flow " + flowM3s + " m3/s outside curve range ["
                    + first.FlowM3s + " m3/s ... " + last.FlowM3s + " m3/s]");
            for (int i = 0; i < Points.Length - 1; i++)
            {
                FanPoint a = Points[i];
                FanPoint b = Points[i + 1];
                if (flowM3s >= a.FlowM3s && flowM3s <= b.FlowM3s)
                {
                    double t = (flowM3s - a.FlowM3s) / (b.FlowM3s - a.FlowM3s);
                    return a.StaticPressurePa + t * (b.StaticPressurePa - a.StaticPressurePa);
                }
            }
            throw new WentaException("fan '" + Name + "': no segment brackets flow " + flowM3s + " m3/s");
        }

        internal static bool IsFinite(double v)
        {
            return !double.IsNaN(v) && !double.IsInfinity(v);
        }
    }

    public static class Fan
    {
        public static double Power(double flowM3s, double pressurePa, double efficiency)
        {
            if (!FanCurve.IsFinite(flowM3s) || !FanCurve.IsFinite(pressurePa)
                || !FanCurve.IsFinite(efficiency))
                throw new WentaException("fan_power: flow, pressure and efficiency must be finite");
            if (flowM3s < 0.0)
                throw new WentaException("fan_power: flow must be >= 0 m3/s, got " + flowM3s);
            if (pressurePa < 0.0)
                throw new WentaException("fan_power: pressure must be >= 0 Pa, got " + pressurePa);
            if (efficiency <= 0.0 || efficiency > 1.0)
                throw new WentaException("fan_power: efficiency must be in (0, 1], got " + efficiency);
            return flowM3s * pressurePa / efficiency;
        }

        public static double? Margin(FanCurve fan, double designFlowM3s, double requiredStaticPa)
        {
            if (!FanCurve.IsFinite(designFlowM3s) || designFlowM3s < 0.0)
                throw new WentaException("margin_pa: design flow must be a finite value >= 0 m3/s, got "
                    + designFlowM3s);
            if (!FanCurve.IsFinite(requiredStaticPa) || requiredStaticPa < 0.0)
                throw new WentaException("margin_pa: required pressure must be a finite value >= 0 Pa, got "
                    + requiredStaticPa);
            if (designFlowM3s < fan.Points[0].FlowM3s
                || designFlowM3s > fan.Points[fan.Points.Length - 1].FlowM3s)
                return null;
            return fan.StaticPressureAt(designFlowM3s) - requiredStaticPa;
        }

        public static int? PickFan(FanCurve[] curves, double designFlowM3s, double requiredStaticPa)
        {
            for (int i = 0; i < curves.Length; i++)
            {
                double? margin = Margin(curves[i], designFlowM3s, requiredStaticPa);
                if (margin.HasValue && margin.Value >= 0.0)
                    return i;
            }
            return null;
        }
    }
}
