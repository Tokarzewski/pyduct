using System;

namespace Wenta
{
    public abstract class CrossSection
    {
        public readonly double Area;
        public readonly double HydraulicDiameter;
        public abstract string Describe();
        public abstract double Perimeter { get; }

        protected CrossSection(double area, double hydraulicDiameter)
        {
            Area = area;
            HydraulicDiameter = hydraulicDiameter;
        }
    }

    public sealed class Round : CrossSection
    {
        public readonly double Diameter;

        public Round(double diameter)
            : base(Math.PI * (diameter / 2.0) * (diameter / 2.0), diameter)
        {
            if (diameter <= 0.0)
                throw new WentaException("diameter must be positive, got " + diameter);
            Diameter = diameter;
        }

        public override string Describe() { return "round D=" + (Diameter * 1000) + "mm"; }
        public override double Perimeter { get { return Math.PI * Diameter; } }
    }

    public sealed class Rectangular : CrossSection
    {
        public readonly double Width;
        public readonly double Height;

        public Rectangular(double width, double height)
            : base(width * height, 2.0 * width * height / (width + height))
        {
            if (width <= 0.0 || height <= 0.0)
                throw new WentaException(
                    "width and height must be positive, got width=" + width
                    + ", height=" + height);
            Width = width;
            Height = height;
        }

        public override string Describe()
        {
            return "rect " + (Width * 1000) + "x" + (Height * 1000) + "mm";
        }
        public override double Perimeter { get { return 2.0 * (Width + Height); } }
    }

    public struct Point2
    {
        public readonly double X;
        public readonly double Y;

        public Point2(double x, double y)
        {
            X = x;
            Y = y;
        }

        public double DistanceSquaredTo(Point2 other)
        {
            double dx = X - other.X;
            double dy = Y - other.Y;
            return dx * dx + dy * dy;
        }

        public double DistanceTo(Point2 other)
        {
            return Math.Sqrt(DistanceSquaredTo(other));
        }
    }

    public class DuctSegment
    {
        public readonly string ComponentId;
        public readonly Point2 Start;
        public readonly Point2 End;
        public readonly double Diameter;

        public DuctSegment(string componentId, Point2 start, Point2 end, double diameter)
        {
            ComponentId = componentId;
            Start = start;
            End = end;
            Diameter = diameter;
        }
    }

    public static class Geometry
    {
        public static double EquivalentRoundDiameter(double width, double height)
        {
            if (width <= 0.0 || height <= 0.0)
                throw new WentaException("width and height must be positive");
            return 1.30 * Math.Pow(width * height, 0.625) / Math.Pow(width + height, 0.25);
        }
    }
}
