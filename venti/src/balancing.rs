//! System balancing — compute the damper loss coefficient (ζ) and setting each
//! branch needs to hit its target terminal flow, mirroring CADvent / VentPack
//! balancing.
//!
//! The routine mirrors how balancing dampers are sized in practice:
//!
//! 1. Each terminal/target has a **required pressure drop** `dp_req` (set by the
//!    designer). The pressure actually available at the branch is
//!    `dp_avail` (computed by the network solver critical-path DP).
//! 2. If `dp_req > dp_avail` the branch is over-supplied and needs a damper to
//!    *eat* the leftover pressure: `Δ = dp_req − dp_avail`.
//! 3. That surplus is converted into a damper loss coefficient via the
//!    dynamic-pressure relation `Δ = ζ · (ρ v² / 2)`, i.e. `ζ = 2Δ/(ρ v²)`.
//! 4. The damper angle/setting is recovered by inverting the butterfly-damper
//!    correlation `ζ = 0.1 + (1 − open/100)² · 10` (below ~95 % open).

/// Loss coefficient (ζ) required of a damper to add exactly `required_dp_pa`
/// of pressure drop at the given velocity — the extra drop the damper must
/// produce.
///
/// ```text
/// dp = ζ · (ρ v² / 2)   →   ζ = 2·dp / (ρ v²)
/// ```
#[inline]
pub fn required_zeta(required_dp_pa: f64, velocity: f64, density: f64) -> f64 {
    let dynamic = density * velocity * velocity * 0.5;
    if dynamic <= 0.0 {
        return 0.0;
    }
    required_dp_pa / dynamic
}

/// Damper ζ that balances a branch whose *available* pressure is below its
/// total required pressure.
///
/// ```text
/// Δ = total_req_pa − branch_available_pa     (the surplus the damper must eat)
/// ζ = 2·Δ / (ρ v²)
/// ```
///
/// Returns `0.0` when the branch already meets its requirement (available ≥
/// required) — the damper is left fully open.
#[inline]
pub fn balancing_zeta(
    total_req_pa: f64,
    branch_available_pa: f64,
    velocity: f64,
    density: f64,
) -> f64 {
    let delta = total_req_pa - branch_available_pa;
    if delta <= 0.0 {
        return 0.0;
    }
    required_zeta(delta, velocity, density)
}

/// Invert the butterfly-damper correlation to recover the open percentage [0,
/// 100] that produces the given ζ.
///
/// ```text
/// ζ = 0.1 + (1 − open/100)² · 10        (open < 95)
/// ```
/// so
/// ```text
/// open = 100 · (1 − sqrt((ζ − 0.1)/10))
/// ```
/// Returns `100.0` for `ζ ≤ 0.1` (fully open / below the fully-open floor).
/// Returns `0.0` for `ζ` at/beyond the fully-closed ceiling (~10.1).
#[inline]
pub fn damper_open_percentage(zeta: f64) -> f64 {
    if zeta <= 0.1 {
        return 100.0;
    }
    let root = ((zeta - 0.1) / 10.0).sqrt();
    let open = 100.0 * (1.0 - root);
    open.clamp(0.0, 100.0)
}

/// Per-branch balancing: given each branch's required and available pressure
/// plus its velocity, return the ζ needed of each branch's damper.
///
/// Returns a `Vec` aligned with the input slices. If the inputs differ in
/// length the shorter wins (pairs are zip-iterated).
#[inline]
pub fn balancing_zeta_batch(
    total_req_pa: &[f64],
    branch_available_pa: &[f64],
    velocity: &[f64],
    density: f64,
) -> Vec<f64> {
    total_req_pa
        .iter()
        .zip(branch_available_pa.iter())
        .zip(velocity.iter())
        .map(|((req, avail), v)| balancing_zeta(*req, *avail, *v, density))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damper_butterfly;
    use crate::physics::losses::local_pressure_drop;

    #[test]
    fn required_zeta_closed_form() {
        // ζ = 2·dp/(ρ v²). ρ=1.204, v=4 → dynamic = 1.204*16/2 = 9.632.
        let zeta = required_zeta(9.632, 4.0, 1.204);
        assert!((zeta - 1.0).abs() < 1e-12, "zeta = {zeta}");

        // Sanity: the induced drop round-trips through local_pressure_drop.
        let dp = local_pressure_drop(zeta, 4.0, 1.204);
        assert!((dp - 9.632).abs() < 1e-12);

        // Zero dynamic pressure guard.
        assert_eq!(required_zeta(5.0, 0.0, 1.204), 0.0);
    }

    #[test]
    fn round_trip_zeta_damper_open_consistent() {
        // For a range of zetas the recovered open% must reproduce the same ζ.
        for zeta in [0.1, 0.5, 1.0, 2.5, 5.0, 8.0, 10.0] {
            let open = damper_open_percentage(zeta);
            let back = damper_butterfly(open).unwrap();
            assert!(
                (back - zeta).abs() < 1e-9,
                "zeta={zeta} open={open} back={back}"
            );
        }
        // Below the fully-open floor → fully open → still ζ = 0.1.
        assert_eq!(damper_open_percentage(0.05), 100.0);
        // Fully-open round trip.
        assert!(damper_butterfly(damper_open_percentage(0.1)).unwrap() - 0.1 < 1e-12);
    }

    #[test]
    fn balancing_zeta_delta_logic() {
        // Branch short by 19.264 Pa at dynamic pressure 9.632 → ζ = 2.
        let zeta = balancing_zeta(30.0, 10.736, 4.0, 1.204);
        assert!((zeta - 2.0).abs() < 1e-12, "zeta = {zeta}");

        // Branch already meets requirement → 0 (damper fully open).
        assert_eq!(balancing_zeta(10.0, 20.0, 4.0, 1.204), 0.0);
        // Exactly met → 0.
        assert_eq!(balancing_zeta(10.0, 10.0, 4.0, 1.204), 0.0);

        // The added ζ reproduces the missing drop.
        let v = 4.0;
        let rho = 1.204;
        let req = 30.0;
        let avail = 10.736;
        let z = balancing_zeta(req, avail, v, rho);
        let added = local_pressure_drop(z, v, rho);
        assert!((added - (req - avail)).abs() < 1e-9);
    }

    #[test]
    fn batch_balances_each_branch() {
        let req = [30.0, 10.0];
        let avail = [10.736, 20.0];
        let vel = [4.0, 4.0];
        let zetas = balancing_zeta_batch(&req, &avail, &vel, 1.204);
        assert!((zetas[0] - 2.0).abs() < 1e-12);
        assert_eq!(zetas[1], 0.0);
    }
}
