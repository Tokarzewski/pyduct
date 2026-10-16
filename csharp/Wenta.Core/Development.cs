using System;

namespace Wenta
{
    public static class Development
    {
        private static void RequirePositive(double value, string name)
        {
            if (double.IsNaN(value) || double.IsInfinity(value))
                throw new WentaException(name + " must be finite, got " + value);
            if (value <= 0.0)
                throw new WentaException(name + " must be positive, got " + value);
        }

        public static FlatPiece RoundDuctDevelopment(double ductDiameterM, double lengthM)
        {
            RequirePositive(ductDiameterM, "duct_diameter_m");
            RequirePositive(lengthM, "length_m");
            double width = Math.PI * ductDiameterM;
            return new FlatPiece(width, lengthM);
        }

        public static FlatPiece RoundElbowDevelopment(double radiusM, double diameterM,
            double angleDeg, uint segments)
        {
            RequirePositive(radiusM, "radius_m");
            RequirePositive(diameterM, "diameter_m");
            RequirePositive(angleDeg, "angle_deg");
            if (segments == 0)
                throw new WentaException("segments must be at least 1");

            double angleRad = angleDeg * Math.PI / 180.0;
            double width = Math.PI * diameterM;
            double arcLength = angleRad * radiusM;
            return new FlatPiece(width, arcLength);
        }

        public static FlatPiece ReducerConeDevelopment(double dSmallM, double dLargeM,
            double lengthM)
        {
            RequirePositive(dSmallM, "d_small_m");
            RequirePositive(dLargeM, "d_large_m");
            RequirePositive(lengthM, "length_m");
            double halfDelta = Math.Abs(dLargeM - dSmallM) / 2.0;
            double slant = Math.Sqrt(lengthM * lengthM + halfDelta * halfDelta);
            double width = Math.PI * (dSmallM + dLargeM) / 2.0;
            return new FlatPiece(width, slant);
        }
    }

    public struct FlatPiece
    {
        public readonly double WidthM;
        public readonly double LengthM;
        public readonly double AreaM2;

        public FlatPiece(double widthM, double lengthM)
        {
            WidthM = widthM;
            LengthM = lengthM;
            AreaM2 = widthM * lengthM;
        }
    }
}
