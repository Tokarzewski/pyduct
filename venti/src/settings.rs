//! Project-level defaults and standards persistence.
//!
//! A ductwork design project bundles a set of decisions that apply across the
//! whole job: which sizing [`Standard`](crate::standards::Standard) to size to,
//! the default duct diameter, absolute roughness, target velocity and friction
//! rate, the noise criterion space, and the display units. These are captured
//! in [`ProjectSettings`].
//!
//! The core struct and [`Units`] enum are dependency-free. The JSON
//! (de)serialization helpers live behind the `cli` feature, matching the
//! pattern used elsewhere in the crate (e.g. [`crate::catalog`]).

use crate::standards::Standard;
use crate::Result;

/// The display/engineering unit system for a project.
///
/// * [`Units::Si`] — metric (mm, m, m/s, Pa).
/// * [`Units::Ip`] — Imperial / customary (in, ft, fpm, in w.c.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub enum Units {
    /// SI / metric units.
    Si,
    /// Imperial (inch-pound) customary units.
    Ip,
}

/// Project-scoped defaults for sizing and pressure-drop.
///
/// These are the values a new project starts from and that every sizing and
/// pressure-drop call uses unless overridden. See also the JSON helpers
/// [`settings_to_json`] and [`settings_from_json`] (available with the `cli`
/// feature) for persisting a project's settings to disk.
///
/// # Examples
///
/// ```
/// use venti::ProjectSettings;
///
/// // A fresh project starts from sensible, valid defaults.
/// let s = ProjectSettings::default();
/// assert!(s.validate().is_ok());
/// assert_eq!(s.standard, venti::standards::Standard::En1505_1506);
/// assert_eq!(s.units, venti::Units::Si);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectSettings {
    /// The sizing standard (dimension set) to use.
    pub standard: Standard,
    /// Default round-duct diameter used when no section is given, in mm.
    pub default_diameter_mm: f64,
    /// Default absolute roughness of the duct material, in metres.
    pub absolute_roughness_m: f64,
    /// Default target duct velocity, in m/s.
    pub target_velocity_ms: f64,
    /// Default target specific pressure drop, in Pa/m.
    pub target_pa_per_m: f64,
    /// Acoustic design space (e.g. `"office"`, `"conference"`, `"lobby"`).
    pub noise_space: String,
    /// The unit system to display results in.
    pub units: Units,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        ProjectSettings {
            standard: Standard::En1505_1506,
            default_diameter_mm: 200.0,
            absolute_roughness_m: 0.0001,
            target_velocity_ms: 4.0,
            target_pa_per_m: 1.0,
            noise_space: "office".to_string(),
            units: Units::Si,
        }
    }
}

impl ProjectSettings {
    /// Validate that every numeric default is physically meaningful.
    ///
    /// Returns `Ok(())` when the settings are valid, otherwise a descriptive
    /// [`Error`](crate::Error). The [`Units`] variant is valid by construction
    /// (the enum only has `Si` and `Ip`, so there is no invalid state).
    pub fn validate(&self) -> Result<()> {
        if self.default_diameter_mm <= 0.0 {
            return Err("default_diameter_mm must be > 0".into());
        }
        if self.absolute_roughness_m <= 0.0 {
            return Err("absolute_roughness_m must be > 0".into());
        }
        if self.target_velocity_ms <= 0.0 {
            return Err("target_velocity_ms must be > 0".into());
        }
        if self.target_pa_per_m <= 0.0 {
            return Err("target_pa_per_m must be > 0".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// JSON persistence (serde, behind the `cli` feature)
// ---------------------------------------------------------------------------

// `Standard` lives in `crate::standards` and must not gain serde derives there;
// we provide the serde impls here instead (the type is local to this crate, so
// the orphan rule permits implementing foreign traits for it). It round-trips
// through its variant name as a string.

#[cfg(feature = "cli")]
impl serde::Serialize for Standard {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Standard::En1505_1506 => "En1505_1506",
            Standard::AsHrae => "AsHrae",
            Standard::Din => "Din",
        })
    }
}

#[cfg(feature = "cli")]
impl<'de> serde::Deserialize<'de> for Standard {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        match name.as_str() {
            "En1505_1506" => Ok(Standard::En1505_1506),
            "AsHrae" => Ok(Standard::AsHrae),
            "Din" => Ok(Standard::Din),
            other => Err(serde::de::Error::custom(format!(
                "unknown standard: {other}"
            ))),
        }
    }
}

/// Serialize [`ProjectSettings`] to a JSON string.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use venti::{settings_to_json, ProjectSettings};
///
/// let s = ProjectSettings::default();
/// let json = settings_to_json(&s).unwrap();
/// assert!(json.contains(r#""standard":"En1505_1506""#));
/// # }
/// ```
#[cfg(feature = "cli")]
pub fn settings_to_json(s: &ProjectSettings) -> Result<String> {
    Ok(serde_json::to_string(s).map_err(|e| format!("settings JSON: {e}"))?)
}

/// Parse [`ProjectSettings`] from a JSON string and validate them.
///
/// # Examples
///
/// ```
/// use venti::{settings_from_json, ProjectSettings};
///
/// let s = settings_from_json(
///     r#"{ "standard": "AsHrae", "default_diameter_mm": 250.0,
///          "absolute_roughness_m": 0.00015, "target_velocity_ms": 5.5,
///          "target_pa_per_m": 1.2, "noise_space": "conference",
///          "units": "Ip" }"#,
/// ).unwrap();
///
/// assert_eq!(s.standard, venti::standards::Standard::AsHrae);
/// assert_eq!(s.units, venti::Units::Ip);
/// assert_eq!(s.default_diameter_mm, 250.0);
/// assert!(s.validate().is_ok());
/// ```
#[cfg(feature = "cli")]
pub fn settings_from_json(json: &str) -> Result<ProjectSettings> {
    let s: ProjectSettings =
        serde_json::from_str(json).map_err(|e| format!("settings JSON: {e}"))?;
    s.validate()?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let s = ProjectSettings::default();
        assert!(s.validate().is_ok());
    }

    #[test]
    fn validate_rejects_non_positive_values() {
        assert!(ProjectSettings {
            default_diameter_mm: 0.0,
            ..ProjectSettings::default()
        }
        .validate()
        .is_err());

        assert!(ProjectSettings {
            absolute_roughness_m: -1.0,
            ..ProjectSettings::default()
        }
        .validate()
        .is_err());

        assert!(ProjectSettings {
            target_velocity_ms: 0.0,
            ..ProjectSettings::default()
        }
        .validate()
        .is_err());

        assert!(ProjectSettings {
            target_pa_per_m: -0.5,
            ..ProjectSettings::default()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn units_default_is_si() {
        assert_eq!(ProjectSettings::default().units, Units::Si);
        assert_ne!(ProjectSettings::default().units, Units::Ip);
    }

    #[test]
    fn standard_default_is_en() {
        assert_eq!(ProjectSettings::default().standard, Standard::En1505_1506);
    }

    #[test]
    fn standard_round_trips() {
        for st in [Standard::En1505_1506, Standard::AsHrae, Standard::Din] {
            let s = ProjectSettings {
                standard: st,
                ..ProjectSettings::default()
            };
            // JSON round-trip is covered by the cli-gated tests; here we just
            // confirm the in-memory settings carry the requested standard.
            assert_eq!(s.standard, st);
        }
    }

    #[cfg(feature = "cli")]
    #[test]
    fn json_round_trip_preserves_all_fields() {
        let s = ProjectSettings {
            standard: Standard::Din,
            default_diameter_mm: 315.0,
            absolute_roughness_m: 0.00015,
            target_velocity_ms: 5.5,
            target_pa_per_m: 1.2,
            noise_space: "conference".to_string(),
            units: Units::Ip,
        };
        let json = settings_to_json(&s).unwrap();
        let back = settings_from_json(&json).unwrap();
        assert_eq!(back, s);

        // Also confirm from the explicit textual form.
        let parsed = settings_from_json(
            r#"{ "standard": "AsHrae", "default_diameter_mm": 250.0,
                 "absolute_roughness_m": 0.00015, "target_velocity_ms": 5.5,
                 "target_pa_per_m": 1.2, "noise_space": "lobby",
                 "units": "Ip" }"#,
        )
        .unwrap();
        assert_eq!(parsed.standard, Standard::AsHrae);
        assert_eq!(parsed.units, Units::Ip);
        assert_eq!(parsed.noise_space, "lobby");
        assert_eq!(parsed.default_diameter_mm, 250.0);
    }

    #[cfg(feature = "cli")]
    #[test]
    fn json_from_rejects_invalid_values() {
        let json = r#"{ "standard": "En1505_1506", "default_diameter_mm": 0.0,
                       "absolute_roughness_m": 0.0001, "target_velocity_ms": 4.0,
                       "target_pa_per_m": 1.0, "noise_space": "office",
                       "units": "Si" }"#;
        assert!(settings_from_json(json).is_err());
    }
}
