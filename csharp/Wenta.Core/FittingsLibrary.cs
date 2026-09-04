using System;

namespace Wenta
{
    /// <summary>Loss-coefficient correlations for common HVAC fittings.
    /// Port of `wentamojo.components.fittings_library` — ASHRAE Fundamentals,
    /// Hendiger, Idelchik.</summary>
    public static class FittingsLibrary
    {
        /// <summary>Round reducer (ASHRAE correlation).
        /// zeta ≈ 0.04 + 0.37·(1 − A_out/A_in), referenced to the outlet
        /// velocity; angle factor softens below 45°.</summary>
        public static double ReducerRound(double dInlet, double dOutlet, double angleDeg = 45.0)
        {
            if (dOutlet > dInlet)
                throw new WentaException("outlet diameter must be <= inlet");
            if (dOutlet <= 0.0)
                throw new WentaException("outlet diameter must be positive");
            double areaRatio = (dOutlet / dInlet) * (dOutlet / dInlet);
            double zeta = 0.04 + 0.37 * (1.0 - areaRatio);
            double angleFactor = angleDeg < 45.0 ? 0.8 + 0.004 * (45.0 - angleDeg) : 1.0;
            return zeta * angleFactor;
        }

        /// <summary>Round expander / diffuser. Borda–Carnot sudden-enlargement
        /// baseline scaled by a piecewise diffuser factor; referenced to the
        /// inlet velocity.</summary>
        public static double ExpanderRound(double dInlet, double dOutlet, double angleDeg = 45.0)
        {
            if (dInlet > dOutlet)
                throw new WentaException("inlet diameter must be <= outlet");
            if (dInlet <= 0.0)
                throw new WentaException("inlet diameter must be positive");
            double areaRatio = (dInlet / dOutlet) * (dInlet / dOutlet);
            double zetaSudden = (1.0 - areaRatio) * (1.0 - areaRatio);
            double diffuserFactor;
            if (angleDeg <= 10.0) diffuserFactor = 0.5;
            else if (angleDeg <= 20.0) diffuserFactor = 0.6;
            else if (angleDeg <= 45.0) diffuserFactor = 0.8;
            else diffuserFactor = 1.0;
            return diffuserFactor * zetaSudden;
        }

        /// <summary>(zeta_main, zeta_branch) for a splitting tee.</summary>
        public static void JunctionTeeBranch(double dMain, double dBranch,
            double flowrateMain, double flowrateBranch, out double zetaMain, out double zetaBranch)
        {
            if (flowrateMain < 0.0 || flowrateBranch < 0.0)
                throw new WentaException("flowrates must be non-negative");
            double total = flowrateMain + flowrateBranch;
            if (total <= 0.0)
                throw new WentaException("at least one flowrate must be positive");
            double split = flowrateBranch / total;
            double area = dMain > 0.0 ? (dBranch / dMain) * (dBranch / dMain) : 0.0;
            zetaMain = 0.08 * split + 0.05 * area;
            zetaBranch = 0.3 + 0.5 * (1.0 - area) + 0.4 * split;
        }

        /// <summary>(zeta_main, zeta_branch) for a combining tee — larger
        /// constants than splitting; use for return-air plenums.</summary>
        public static void JunctionTeeCombine(double dMain, double dBranch,
            double flowrateMain, double flowrateBranch, out double zetaMain, out double zetaBranch)
        {
            double total = flowrateMain + flowrateBranch;
            if (total <= 0.0)
                throw new WentaException("at least one flowrate must be positive");
            double split = flowrateBranch / total;
            double area = dMain > 0.0 ? (dBranch / dMain) * (dBranch / dMain) : 0.0;
            zetaMain = 0.1 + 0.15 * split + 0.08 * area;
            zetaBranch = 0.4 + 0.6 * (1.0 - area) + 0.3 * split;
        }

        /// <summary>Butterfly-damper loss coefficient (0–100 % open).
        /// ~0.1 fully open; zeta ≈ 0.1 + (1 − open)² · 10.</summary>
        public static double DamperButterfly(double openPercentage = 100.0)
        {
            if (openPercentage < 0.0 || openPercentage > 100.0)
                throw new WentaException("open_percentage must be in [0, 100]");
            if (openPercentage >= 95.0)
                return 0.1;
            double closedFrac = 1.0 - openPercentage / 100.0;
            return 0.1 + closedFrac * closedFrac * 10.0;
        }

        /// <summary>Ceiling-diffuser face-velocity loss coefficient:
        /// zeta = 0.4 / area_throw.</summary>
        public static double DiffuserCeiling(double areaThrow = 1.0)
        {
            if (areaThrow <= 0.0)
                throw new WentaException("area_throw must be positive");
            return 0.4 / areaThrow;
        }

        /// <summary>Return-grille loss coefficient: zeta = 0.25·(1 + blockage).</summary>
        public static double GrilleReturn(double blockageFactor = 0.15)
        {
            if (blockageFactor < 0.0 || blockageFactor > 1.0)
                throw new WentaException("blockage_factor must be in [0, 1]");
            return 0.25 * (1.0 + blockageFactor);
        }

        /// <summary>Smooth-radius rectangular elbow (Idelchik §6) with aspect
        /// correction: zeta_90 ≈ 0.21/(r/W)^0.5 capped at 1.5, ×(H/W)^0.25,
        /// angle scales linearly off 90°.</summary>
        public static double RectangularElbow(double width, double height,
            double bendRadius, double angleDeg = 90.0)
        {
            double smallest = width <= height ? width : height;
            smallest = bendRadius < smallest ? bendRadius : smallest;
            if (smallest <= 0.0)
                throw new WentaException("width, height and bend_radius must be positive");
            if (angleDeg <= 0.0 || angleDeg > 180.0)
                throw new WentaException("angle_deg must be in (0, 180]");
            double rOverW = bendRadius / width;
            double floor = rOverW > 0.1 ? rOverW : 0.1;
            double zeta90 = 0.21 / Math.Pow(floor, 0.5);
            if (zeta90 > 1.5)
                zeta90 = 1.5;
            double aspectCorrection = Math.Pow(height / width, 0.25);
            return zeta90 * aspectCorrection * (angleDeg / 90.0);
        }

        /// <summary>Sharp-corner mitered elbow; quadratic fit to ASHRAE points
        /// (≤5 % error at 45/60/90/120°). vaned=true cuts the loss to ~40 %.</summary>
        public static double MiteredElbow(double angleDeg = 90.0, bool vaned = false)
        {
            if (angleDeg <= 0.0 || angleDeg > 180.0)
                throw new WentaException("angle_deg must be in (0, 180]");
            double a = angleDeg / 90.0;
            double zetaUnvaned = 0.55 * a + 0.65 * a * a;
            return zetaUnvaned * (vaned ? 0.4 : 1.0);
        }
    }
}
