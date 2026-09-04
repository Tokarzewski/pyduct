using System;
using System.Collections.Generic;

namespace Wenta
{
    /// <summary>Duct sizing methods. Port of `wenta.sizing`: velocity,
    /// equal-friction, pressure-drop budget, noise (NC) limit, aspect-ratio.
    /// All methods return the smallest EN-standard section meeting target.</summary>
    public static class Sizing
    {
        public const string ShapeRound = "round";
        public const string ShapeRectangular = "rectangular";

        /// <summary>ASHRAE-style maximum air velocity by space type [m/s].</summary>
        public static readonly Dictionary<string, double> NoiseLimitsMs =
            new Dictionary<string, double>
        {
            { "studio", 2.5 },     // recording / broadcast
            { "bedroom", 3.0 },
            { "office", 4.0 },
            { "classroom", 4.5 },
            { "retail", 5.0 },
            { "industrial", 7.5 },
        };

        public sealed class SizingResult
        {
            public CrossSection Section;
            public double Velocity;
            public double PressureDropPerMeter; // equal-friction / budget only
        }

        private static CrossSection[] SectionsFor(string shape)
        {
            if (shape == ShapeRound) return StandardSizes.RoundSections();
            if (shape == ShapeRectangular) return StandardSizes.RectangularSections();
            throw new WentaException("unknown shape '" + shape + "'");
        }

        /// <summary>Return (section, value) for the smallest section whose
        /// evaluated value is ≤ target; fall back to the largest section.</summary>
        private static void SmallestMeeting(CrossSection[] sections,
            Func<CrossSection, double> evaluator, double target,
            out CrossSection section, out double value)
        {
            CrossSection lastSection = null;
            double lastValue = 0.0;
            foreach (CrossSection s in sections)
            {
                lastValue = evaluator(s);
                lastSection = s;
                if (lastValue <= target) { section = s; value = lastValue; return; }
            }
            if (lastSection == null)
                throw new WentaException("no standard sections available");
            section = lastSection;
            value = lastValue;
        }

        /// <summary>Size a duct so velocity ≤ target_velocity (default 4 m/s).</summary>
        public static SizingResult VelocityMethod(double flowrate, string shape = ShapeRound,
            double targetVelocity = 4.0)
        {
            if (flowrate <= 0.0)
                throw new WentaException("flowrate must be positive, got " + flowrate);
            if (targetVelocity <= 0.0)
                throw new WentaException("target_velocity must be positive, got " + targetVelocity);
            CrossSection s; double v;
            SmallestMeeting(SectionsFor(shape),
                delegate(CrossSection sec) { return flowrate / sec.Area; },
                targetVelocity, out s, out v);
            return new SizingResult { Section = s, Velocity = v };
        }

        /// <summary>Size a duct so linear pressure drop ≤ target (default 1 Pa/m).</summary>
        public static SizingResult EqualFrictionMethod(double flowrate,
            double targetPressureDropPerMeter = 1.0, string shape = ShapeRound,
            double absoluteRoughness = 0.0001, Fluid fluid = null)
        {
            if (flowrate <= 0.0)
                throw new WentaException("flowrate must be positive, got " + flowrate);
            if (targetPressureDropPerMeter <= 0.0)
                throw new WentaException(
                    "target_pressure_drop_per_meter must be positive, got "
                    + targetPressureDropPerMeter);
            fluid = fluid ?? Fluid.StandardAir();
            double nu = fluid.KinematicViscosity;
            double rho = fluid.Density;

            CrossSection s; double dp;
            SmallestMeeting(SectionsFor(shape),
                delegate(CrossSection sec)
                {
                    double v = flowrate / sec.Area;
                    double dh = sec.HydraulicDiameter;
                    double f = Friction.FrictionFactor(
                        Friction.Reynolds(v, dh, nu),
                        Friction.RelativeRoughness(absoluteRoughness, dh));
                    return f / dh * (rho * v * v) / 2.0;
                },
                targetPressureDropPerMeter, out s, out dp);
            return new SizingResult
            {
                Section = s,
                Velocity = flowrate / s.Area,
                PressureDropPerMeter = dp,
            };
        }

        /// <summary>Size a duct so total drop across length ≤ budget_pa.</summary>
        public static SizingResult PressureDropBudget(double flowrate, double length,
            double budgetPa, string shape = ShapeRound,
            double absoluteRoughness = 0.0001, Fluid fluid = null)
        {
            if (length <= 0.0)
                throw new WentaException("length must be positive, got " + length);
            if (budgetPa <= 0.0)
                throw new WentaException("budget_pa must be positive, got " + budgetPa);
            return EqualFrictionMethod(flowrate, budgetPa / length, shape,
                absoluteRoughness, fluid);
        }

        /// <summary>Size a duct for the maximum velocity allowed by the
        /// space type's noise criterion (NC).</summary>
        public static SizingResult NoiseLimitMethod(double flowrate, string spaceType,
            string shape = ShapeRound, double absoluteRoughness = 0.0001,
            Fluid fluid = null)
        {
            if (!NoiseLimitsMs.ContainsKey(spaceType))
                throw new WentaException("unknown space_type '" + spaceType + "'");
            return VelocityMethod(flowrate, shape, NoiseLimitsMs[spaceType]);
        }

        /// <summary>Size a rectangular duct for a target velocity at a given
        /// aspect ratio. Iterates flattest-first candidates.</summary>
        public static SizingResult AspectRatioMethod(double flowrate,
            double targetVelocity = 4.0, double aspectRatio = 2.0)
        {
            if (flowrate <= 0.0)
                throw new WentaException("flowrate must be positive, got " + flowrate);
            if (targetVelocity <= 0.0)
                throw new WentaException("target_velocity must be positive, got " + targetVelocity);
            if (aspectRatio < 1.0)
                throw new WentaException("aspect_ratio must be >= 1, got " + aspectRatio);

            var candidates = new List<Rectangular>();
            foreach (Rectangular s in StandardSizes.RectangularSections())
            {
                // integer-millimetre comparison: avoids 0.3/0.1 = 2.999... fp traps
                double maxMm = Math.Round(Math.Max(s.Width, s.Height) * 1000.0);
                double minMm = Math.Round(Math.Min(s.Width, s.Height) * 1000.0);
                if (maxMm / minMm >= aspectRatio)
                    candidates.Add(s);
            }
            if (candidates.Count == 0)
                throw new WentaException(
                    "no standard rectangular size meets aspect_ratio=" + aspectRatio);
            candidates.Sort(delegate(Rectangular a, Rectangular b)
            {
                return a.Area.CompareTo(b.Area);
            });
            foreach (Rectangular s in candidates)
            {
                double v = flowrate / s.Area;
                if (v <= targetVelocity)
                    return new SizingResult { Section = s, Velocity = v };
            }
            Rectangular last = candidates[candidates.Count - 1];
            return new SizingResult { Section = last, Velocity = flowrate / last.Area };
        }
    }
}
