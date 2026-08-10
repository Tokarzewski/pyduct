//! Sound / acoustics for ductwork: airflow-regenerated noise, the room
//! equation that converts duct sound power into a space sound-pressure
//! level, and NC (Noise Criterion) compliance checks.
//!
//! Mirrors the "calculation of sound" feature of CADvent / duct-design
//! practice: a duct carrying air generates noise (regenerated noise) that
//! depends strongly on velocity and on duct size; that sound power is then
//! attenuated by the duct path and finally radiates into the served room,
//! where it must stay below the space's NC target.
//!
//! This module is **dependency-free** — pure `f64` math on standard Rust.

use crate::core::fluid::STANDARD_AIR;

/// Reference air density used to normalise the density term of the
/// regenerated-noise correlation (kg/m³, standard air at 20 °C).
const RHO_0: f64 = STANDARD_AIR.density;

/// Calibration offset for the regenerated-noise correlation (dB re 1e-12 W).
const REGEN_C: f64 = 10.0;

// ---------------------------------------------------------------------------
// Regenerated noise
// ---------------------------------------------------------------------------

/// Regenerated (airflow) sound-power level of a straight round duct, in dB
/// re 1e-12 W.
///
/// # Reference formula
///
/// Turbulent airflow in a duct radiates acoustic power that scales with the
/// aerodynamic power of the flow: `W ∝ ρ·v⁶` for the velocity (Lighthill's
/// v⁶ law for subsonic aerodynamic sound) and falls as the duct grows, so
/// that for a *fixed* velocity a larger duct generates less noise (the same
/// turbulent energy is spread over a larger, thinner boundary layer whose
/// wall-pressure fluctuations couple less efficiently to the acoustic field).
/// Working in dB against the 1e-12 W reference:
///
/// ```text
/// Lw = C + 10·log10(ρ/ρ₀) + 60·log10(v) − 20·log10(d)
/// ```
///
/// with `v` = mean duct velocity [m/s], `d` = duct internal diameter [m],
/// `ρ` = air density [kg/m³] (defaults to standard air), `ρ₀` = 1.204 kg/m³,
/// and `C` = 10 dB (calibration offset putting the level on the usual
/// regenerated-noise range). Hence regenerated noise grows ~ `v⁶` and falls
/// ~ `1/d²` — the physically-motivated behaviour asserted by the unit tests.
///
/// # Arguments
///
/// * `velocity` — mean duct air velocity [m/s], must be `> 0`.
/// * `diameter` — duct internal diameter [m], must be `> 0`.
/// * `density` — optional air density [kg/m³], must be `> 0`. Defaults to
///   [`STANDARD_AIR`].
///
/// # Errors
///
/// Returns an error string for non-positive velocity, diameter, or density.
pub fn regenerated_noise_round(
    velocity: f64,
    diameter: f64,
    density: Option<f64>,
) -> Result<f64, String> {
    if velocity <= 0.0 {
        return Err(format!("velocity must be positive, got {velocity}"));
    }
    if diameter <= 0.0 {
        return Err(format!("diameter must be positive, got {diameter}"));
    }
    let rho = density.unwrap_or(RHO_0);
    if rho <= 0.0 {
        return Err(format!("density must be positive, got {rho}"));
    }

    Ok(REGEN_C
        + 10.0 * (rho / RHO_0).log10()
        + 60.0 * velocity.log10()
        - 20.0 * diameter.log10())
}

// ---------------------------------------------------------------------------
// Duct sound pressure level (room equation)
// ---------------------------------------------------------------------------

/// Convert a duct sound-*power* level [dB re 1e-12 W] into a reverberant
/// sound-*pressure* level [dB re 20 µPa] inside the served room.
///
/// # Reference formula
///
/// Under the diffuse-field assumption, the reverberant sound pressure level
/// produced by a sound power source in a room is given by the classical
/// "room equation" (ISO 3740 family / acoustic room theory):
///
/// ```text
/// Lp = Lw + 10·log10( 4·(1 − α) / (α·S) )
/// ```
///
/// where `Lw` = source sound power level [dB], `S` = total room surface area
/// [m²], and `α` = average Sabine absorption coefficient of the room (an
/// open, absorptive room attenuates the level, a small, reverberant room
/// raises it). This is the transducer that turns duct regenerated noise into
/// the level a person actually hears.
///
/// # Arguments
///
/// * `sound_power_db` — source sound power level [dB re 1e-12 W].
/// * `room_surface_area` — total internal room surface area [m²], `> 0`.
/// * `absorption_coefficient` — average absorption coefficient, in `(0, 1)`.
///
/// # Errors
///
/// Returns an error for a non-positive area, or an absorption coefficient
/// outside the open interval `(0, 1)` (walls that reflect 100% or absorb
/// 100% make the reverberant term undefined).
pub fn duct_pressure_level(
    sound_power_db: f64,
    room_surface_area: f64,
    absorption_coefficient: f64,
) -> Result<f64, String> {
    if room_surface_area <= 0.0 {
        return Err(format!(
            "room_surface_area must be positive, got {room_surface_area}"
        ));
    }
    if !(absorption_coefficient > 0.0 && absorption_coefficient < 1.0) {
        return Err(format!(
            "absorption_coefficient must be in (0, 1), got {absorption_coefficient}"
        ));
    }

    let room_term =
        4.0 * (1.0 - absorption_coefficient) / (absorption_coefficient * room_surface_area);
    Ok(sound_power_db + 10.0 * room_term.log10())
}

// ---------------------------------------------------------------------------
// NC (Noise Criterion) compliance
// ---------------------------------------------------------------------------

/// Typical NC (Noise Criterion) targets by space type, dB.
///
/// Parallel to [`crate::sizing::NOISE_LIMITS_M_S`], which caps *velocity* to
/// keep a space quiet; this table caps the resulting *level* directly.
pub const NOISE_LIMITS_NC: &[(&str, f64)] = &[
    ("studio", 25.0), // recording / broadcast
    ("bedroom", 25.0),
    ("office", 35.0),
    ("classroom", 35.0),
    ("retail", 40.0),
    ("industrial", 60.0),
];

fn nc_limit(space_type: &str) -> Result<f64, String> {
    for (k, v) in NOISE_LIMITS_NC {
        if *k == space_type {
            return Ok(*v);
        }
    }
    Err(format!(
        "unknown space_type {space_type:?}; expected one of studio|bedroom|office|classroom|retail|industrial"
    ))
}

/// Check a computed sound level against the NC target for `space_type`.
///
/// Returns `Ok(true)` when `level_db` is at or below the space's NC limit,
/// reusing the [`NOISE_LIMITS_NC`] mapping (parallel to
/// `venti::sizing::NOISE_LIMITS_M_S`).
///
/// # Errors
///
/// Returns an error string for an unknown `space_type`.
pub fn nc_ok(space_type: &str, level_db: f64) -> Result<bool, String> {
    let limit = nc_limit(space_type)?;
    Ok(level_db <= limit + 1e-9)
}

/// Check a computed sound level against an explicit numeric NC target.
///
/// Returns `true` when `level_db` is at or below `nc_target`. Unlike
/// [`nc_ok`] this needs no space-type lookup, so it cannot fail.
pub fn nc_ok_target(nc_target: f64, level_db: f64) -> bool {
    level_db <= nc_target + 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regenerated_noise_grows_with_velocity() {
        let d = 0.2;
        let lb = regenerated_noise_round(2.0, d, None).unwrap();
        let hi = regenerated_noise_round(6.0, d, None).unwrap();
        assert!(hi > lb, "faster air must be louder: {lb} vs {hi}");
        // v⁶ scaling => 6 m/s / 2 m/s = ratio 3 => +60·log10(3) ≈ +28.6 dB.
        assert!((hi - lb - 60.0 * 3.0f64.log10()).abs() < 1e-9);
    }

    #[test]
    fn regenerated_noise_falls_with_diameter() {
        let v = 4.0;
        let small = regenerated_noise_round(v, 0.1, None).unwrap();
        let large = regenerated_noise_round(v, 0.5, None).unwrap();
        assert!(large < small, "bigger duct must be quieter: {small} vs {large}");
        // 1/d² scaling => 0.5/0.1 = ratio 5 => −20·log10(5) ≈ −14 dB.
        assert!((large - small - (-20.0 * 5.0f64.log10())).abs() < 1e-9);
    }

    #[test]
    fn regenerated_noise_closed_form() {
        // With default density the ρ/ρ₀ term vanishes, leaving an exact
        // closed form: Lw = 10 + 60·log10(v) − 20·log10(d).
        let v = 2.0;
        let d = 0.5;
        let lw = regenerated_noise_round(v, d, None).unwrap();
        let expected = 10.0 + 60.0 * v.log10() - 20.0 * d.log10();
        assert!((lw - expected).abs() < 1e-9);
    }

    #[test]
    fn regenerated_noise_higher_density_is_louder() {
        let thin = regenerated_noise_round(4.0, 0.2, Some(1.0)).unwrap();
        let dense = regenerated_noise_round(4.0, 0.2, Some(1.5)).unwrap();
        assert!(dense > thin);
        assert!((dense - thin - 10.0 * (1.5f64 / 1.0f64).log10()).abs() < 1e-9);
    }

    #[test]
    fn regenerated_noise_rejects_bad_inputs() {
        assert!(regenerated_noise_round(0.0, 0.2, None).is_err());
        assert!(regenerated_noise_round(-3.0, 0.2, None).is_err());
        assert!(regenerated_noise_round(4.0, 0.0, None).is_err());
        assert!(regenerated_noise_round(4.0, 0.2, Some(-1.0)).is_err());
    }

    #[test]
    fn duct_pressure_level_closed_form() {
        let s = 100.0;
        let a = 0.2;
        let lw = 60.0;
        let lp = duct_pressure_level(lw, s, a).unwrap();
        let expected = lw + 10.0 * (4.0 * (1.0 - a) / (a * s)).log10();
        assert!((lp - expected).abs() < 1e-9);
    }

    #[test]
    fn duct_pressure_level_more_absorption_is_quieter() {
        let lw = 60.0;
        let s = 100.0;
        let echoey = duct_pressure_level(lw, s, 0.05).unwrap();
        let absorptive = duct_pressure_level(lw, s, 0.45).unwrap();
        assert!(absorptive < echoey, "more absorption must lower SPL");
    }

    #[test]
    fn duct_pressure_level_rejects_bad_inputs() {
        assert!(duct_pressure_level(60.0, 0.0, 0.2).is_err());
        assert!(duct_pressure_level(60.0, 100.0, 0.0).is_err());
        assert!(duct_pressure_level(60.0, 100.0, 1.0).is_err());
        assert!(duct_pressure_level(60.0, -5.0, 0.2).is_err());
    }

    #[test]
    fn nc_ok_boundary() {
        // office NC target = 35 dB: exactly at and under pass, a hair over fails.
        assert!(nc_ok("office", 35.0).unwrap());
        assert!(nc_ok("office", 34.9).unwrap());
        assert!(!nc_ok("office", 35.1).unwrap());
        assert!(!nc_ok("bedroom", 26.0).unwrap());
    }

    #[test]
    fn nc_ok_lookup_and_target() {
        assert_eq!(nc_limit("studio").unwrap(), 25.0);
        assert!(nc_ok("bogus", 10.0).is_err());
        // Numeric target overload agrees with the table lookup.
        assert_eq!(nc_ok_target(35.0, 35.0), nc_ok("office", 35.0).unwrap());
        assert_eq!(nc_ok_target(35.0, 34.0), nc_ok("office", 34.0).unwrap());
        assert_eq!(nc_ok_target(35.0, 36.0), nc_ok("office", 36.0).unwrap());
    }
}
