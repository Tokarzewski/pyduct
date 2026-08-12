//! Regression suite — golden design cases for `venti` (issue #37,
//! ModelTok/pyduct).
//!
//! A curated corpus of representative duct-design calculations with stable
//! reference results. Each case exercises a public `venti::…` API and asserts
//! the outcome within a generous tolerance, so the tests act as regression
//! guards (catching silent drift) without being flaky.
//!
//! Reference values were produced from the closed forms in the module docs
//! (Swamee–Jain friction, Darcy–Weisbach pressure drop, ζ·ρv²/2 local losses,
//! P = Q·p/η fan power, ACH = 3600·Q/V) and cross-checked numerically.

use std::f64::consts::PI;

use venti::{
    elbow_round_loss, equal_friction_method_round, fan_power, room_ach, velocity_method_round,
    ComponentEnum, CrossSection, Network, RigidDuct, Source, Terminal, TwoPortFitting,
    STANDARD_AIR,
};

/// Case 1 — velocity-method sizing of a round duct.
///
/// `velocity_method_round(0.1, 4.0)` must pick the first EN 1506 standard size
/// whose velocity does not exceed 4 m/s. 160 mm runs at 4.97 m/s (over the
/// target), so the 200 mm section is selected and the actual velocity is
/// `Q/A = 0.1 / (π·0.01²)` = 0.1 / (π·0.01).
#[test]
fn velocity_method_round_selected_200mm_at_3p18_ms() {
    let (section, v) =
        velocity_method_round(0.1, 4.0).expect("velocity_method_round(0.1, 4.0) must succeed");
    let expected_v = 0.1 / (PI * 0.01);
    match section {
        CrossSection::Round(r) => {
            assert!(
                (r.diameter - 0.2).abs() < 1e-9,
                "sized diameter = {} m, expected 0.2 m (200 mm)",
                r.diameter
            );
        }
        other => panic!("expected a round section, got {other:?}"),
    }
    assert!(
        v <= 4.0,
        "selected velocity {v} m/s must not exceed the 4 m/s target"
    );
    assert!(
        (v - expected_v).abs() < 1e-6,
        "velocity = {v} m/s, expected {expected_v} m/s (0.1 m³/s through a 200 mm duct)"
    );
}

/// Case 2 — Darcy friction factor (Swamee–Jain) for a typical HVAC duct.
///
/// `friction_factor(5e4, 9e-4)` — Re = 50 000, ε/D = 9·10⁻⁴ — should stay
/// around 0.02364 (the closed-form Swamee–Jain value). Tolerance is wide
/// enough to absorb platform float differences, tight enough to catch an
/// algorithmically wrong friction model.
#[test]
fn friction_factor_swamee_jain_turbulent() {
    let f = venti::physics::friction::friction_factor(5.0e4, 9.0e-4);
    assert!(
        (f - 0.02364).abs() < 1.0e-3,
        "friction_factor(5e4, 9e-4) = {f}, expected ≈ 0.02364"
    );
}

/// Case 3 — end-to-end network solve.
///
/// Chain `Source -> RigidDuct(D=0.2 m, L=20 m, ε=0.0001 m) -> Fitting(ζ=0.5)
/// -> Terminal(0.1 m³/s, ζ=1.0)` with standard air. The critical-path ΔP is
/// the sum of the Darcy–Weisbach duct drop (~14.13 Pa), the fitting local
/// drop (~3.05 Pa) and the terminal device drop (~6.10 Pa) ≈ **23.28 Pa** —
/// computed from the modules, not hard-coded from a stranger run.
#[test]
fn network_critical_path_around_23p28_pa() {
    let r = venti::Round::new(0.2).expect("200 mm round section");
    let mut net = Network::new("regress-chain");
    net.add("ahu", ComponentEnum::Source(Source::new("AHU")))
        .expect("add source");
    net.add(
        "duct",
        ComponentEnum::RigidDuct(
            RigidDuct::new("main duct", r.area, r.hydraulic_diameter, 20.0, 0.0001)
                .expect("20 m rigid duct"),
        ),
    )
    .expect("add duct");
    net.add(
        "fit",
        ComponentEnum::TwoPortFitting(TwoPortFitting::new("elbow ζ=0.5", r.area, 0.5)),
    )
    .expect("add fitting");
    net.add(
        "term",
        ComponentEnum::Terminal(Terminal::new("diffuser", 0.1, Some(r.area), 1.0)),
    )
    .expect("add terminal");
    net.connect("ahu", "duct").expect("ahu -> duct");
    net.connect("duct", "fit").expect("duct -> fit");
    net.connect("fit", "term").expect("fit -> term");

    let dp_pa = net.solve(Some(&STANDARD_AIR)).expect("solve network");
    assert!(
        dp_pa > 23.0 && dp_pa < 24.0,
        "critical-path ΔP = {dp_pa} Pa, expected ≈ 23.28 Pa (23 < ΔP < 24)"
    );
}

/// Case 4 — equal-friction method sizing.
///
/// The returned per-metre drop must never exceed the target (here 1.0 Pa/m
/// for 0.1 m³/s, ε = 0.0001 m, standard air), and must be positive.
#[test]
fn equal_friction_method_respects_target_per_meter() {
    let target = 1.0; // Pa/m
    let (section, v, dp_per_m) = equal_friction_method_round(0.1, target, 0.0001, &STANDARD_AIR)
        .expect("equal_friction_method_round(0.1, 1.0 Pa/m) must succeed");
    assert!(
        dp_per_m <= target,
        "sized ΔP/m = {dp_per_m} Pa/m exceeds target {target} Pa/m"
    );
    assert!(
        dp_per_m > 0.0,
        "sized ΔP/m = {dp_per_m} Pa/m must be positive"
    );
    assert!(v > 0.0, "sized velocity {v} m/s must be positive");
    let name = match section {
        CrossSection::Round(r) => format!("{:.0} mm", r.diameter * 1000.0),
        other => format!("{other:?}"),
    };
    assert!(
        dp_per_m <= target,
        "{name}: ΔP/m = {dp_per_m} Pa/m must stay at or below the {target} Pa/m target"
    );
}

/// Case 5 — insulation thickness against condensation.
///
/// Cold supply air (8 °C) in a 24 °C room at 60 % RH (dew point ≈ 15.8 °C),
/// 200 mm duct, mineral wool λ = 0.035 W/(m·K), indoor film coefficients
/// (h_i = 10, h_e = 8 W/(m²K)). The required thickness is small (≈ 1 mm) and
/// must sit in (0, 0.05) m.
#[test]
fn insulation_condensation_thickness_is_small_and_positive() {
    let t =
        venti::insulation::required_thickness_condensation(8.0, 15.8, 24.0, 0.035, 0.2, 10.0, 8.0)
            .expect("condensation thickness must be computable");
    assert!(
        t > 0.0 && t < 0.05,
        "required insulation = {t} m, expected in (0, 0.05) m"
    );
}

/// Case 6 — fan shaft power at a duty point.
///
/// `fan_power(Q, p, η) = Q·p/η` — 0.5 m³/s against 500 Pa at 60 %
/// efficiency gives 0.5·500/0.6 ≈ 416.67 W.
#[test]
fn fan_power_closed_form() {
    let p = fan_power(0.5, 500.0, 0.6).expect("fan_power must succeed");
    let expected = 0.5 * 500.0 / 0.6;
    assert!(
        (p - expected).abs() < 1.0e-9,
        "fan_power(0.5, 500, 0.6) = {p} W, expected {expected} W (Q·p/η)"
    );
}

/// Case 7 — air changes per hour.
///
/// `room_ach(0.05 m³/s, 50 m³)` = 0.05·3600/50 = 3.6 ACH.
#[test]
fn room_ach_value() {
    let ach = room_ach(0.05, 50.0).expect("room_ach must succeed");
    assert!(
        (ach - 3.6).abs() < 1.0e-9,
        "room_ach(0.05, 50) = {ach} ACH, expected 3.6"
    );
}

/// Case 8 — smooth round-elbow loss coefficient (Re- and size-corrected).
///
/// R/D = 1.0, 90°, 200 mm duct at 4 m/s under standard air: base ζ = 0.21
/// with a mild Reynolds correction (≈ 0.99) gives ≈ 0.208, comfortably inside
/// (0.1, 0.5).
#[test]
fn elbow_round_loss_is_physical() {
    let z = elbow_round_loss(0.2, 0.2, 90.0, 4.0, 1.204, 1.825e-5)
        .expect("elbow_round_loss must succeed");
    assert!(
        z > 0.1 && z < 0.5,
        "elbow_round_loss(0.2, 0.2, 90°, 4 m/s) = {z}, expected in (0.1, 0.5)"
    );
}
