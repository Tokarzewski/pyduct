using System;

namespace Wenta
{
    /// <summary>Cross-section geometry primitives. Port of `wenta.core.geometry`.
    /// Area and hydraulic diameter are computed once at construction.</summary>
    public abstract class CrossSection
    {
        public readonly double Area;             // [m^2]
        public readonly double HydraulicDiameter; // D_h [m]
        public abstract string Describe();

        protected CrossSection(double area, double hydraulicDiameter)
        {
            Area = area;
            HydraulicDiameter = hydraulicDiameter;
        }
    }

    /// <summary>Circular cross-section.</summary>
    public sealed class Round : CrossSection
    {
        public readonly double Diameter; // [m]

        public Round(double diameter)
            : base(Math.PI * (diameter / 2.0) * (diameter / 2.0), diameter)
        {
            if (diameter <= 0.0)
                throw new WentaException("diameter must be positive, got " + diameter);
            Diameter = diameter;
        }

        public override string Describe() { return "round D=" + (Diameter * 1000) + "mm"; }
    }

    /// <summary>Rectangular cross-section.</summary>
    public sealed class Rectangular : CrossSection
    {
        public readonly double Width;  // [m]
        public readonly double Height; // [m]

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
    }

    public static class Geometry
    {
        /// <summary>ASHRAE equivalent round diameter for a rectangular duct:
        /// D_eq = 1.30 · (a·b)^0.625 / (a + b)^0.25 [m].</summary>
        public static double EquivalentRoundDiameter(double width, double height)
        {
            if (width <= 0.0 || height <= 0.0)
                throw new WentaException("width and height must be positive");
            return 1.30 * Math.Pow(width * height, 0.625) / Math.Pow(width + height, 0.25);
        }
    }
}
