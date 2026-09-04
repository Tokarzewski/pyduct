using System;

namespace Wenta
{
    /// <summary>System balancing — damper ζ and setting each branch needs to hit
    /// its target terminal flow (CADvent / VentPack-style). Port of
    /// `venti/src/balancing.rs` (Phase 4 vertical: the "balancing" feature).
    ///
    /// Mirrors how balancing dampers are sized in practice:
    /// 1. Each terminal has a required pressure drop `dp_req` (designer-set);
    ///    available pressure `dp_avail` comes from the network critical path.
    /// 2. If the branch is over-supplied (avail &lt; req) the damper eats the
    ///    surplus `Δ = dp_req − dp_avail`.
    /// 3. Surplus → damper ζ via the dynamic-pressure relation Δ = ζ·(ρ v²/2).
    /// 4. ζ → damper open-% by inverting the butterfly-damper correlation.</summary>
    public static class Balancing
    {
        /// <summary>Damper ζ that produces exactly `requiredDpPa` at `velocity`.
        /// `ζ = 2·dp / (ρ v²)`. Zero for non-positive dynamic pressure.</summary>
        public static double RequiredZeta(double requiredDpPa, double velocity, double density)
        {
            double dynamic = density * velocity * velocity * 0.5;
            if (dynamic <= 0.0) return 0.0;
            return requiredDpPa / dynamic;
        }

        /// <summary>Damper ζ balancing a branch whose available pressure is
        /// below its total required pressure:
        /// `Δ = totalReq − branchAvail`, `ζ = 2·Δ / (ρ v²)`.
        /// Returns 0.0 when the branch already meets its requirement (damper
        /// fully open).</summary>
        public static double BalancingZeta(double totalReqPa, double branchAvailablePa,
            double velocity, double density)
        {
            double delta = totalReqPa - branchAvailablePa;
            if (delta <= 0.0) return 0.0;
            return RequiredZeta(delta, velocity, density);
        }

        /// <summary>Invert the butterfly-damper correlation `ζ = 0.1 + (1−open/100)²·10`
        /// (open &lt; 95) to recover the open percentage in [0, 100].
        /// Returns 100 for ζ ≤ 0.1 (fully open / below fully-open floor);
        /// returns 0 for ζ at/beyond the fully-closed ceiling (~10.1).</summary>
        public static double DamperOpenPercentage(double zeta)
        {
            if (zeta <= 0.1) return 100.0;
            double root = Math.Sqrt((zeta - 0.1) / 10.0);
            double open = 100.0 * (1.0 - root);
            return Math.Max(0.0, Math.Min(100.0, open));
        }
    }
}