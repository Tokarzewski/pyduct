//! A data-driven catalog of duct-fitting loss coefficients (ζ) — FR-19.
//!
//! The library's correlations compute ζ from first principles; this module is
//! the **database layer**: a lookup of published, documented constant ζ values
//! for common fittings and sizes, sourced from the Tier-1/2 references in
//! [`docs/ZETA-SOURCES.md`](crate::catalog) (ASHRAE Fundamentals, SMACNA Duct
//! Design, Idelchik, CIBSE Guide B).
//!
//! * [`ZetaCatalog`] — dependency-free constant ζ lookup, plus a built-in
//!   [`reference_catalog`] populated from the sources.
//! * Vendor JSON catalogues (serde, behind the `cli` feature) load into the
//!   same structure and merge into lookups — see [`vendor_catalog_from_json`].
//!
//! Every entry carries its **category**, the **reference velocity** the ζ is
//! referred to, and its **source** — the three things a ζ value is meaningless
//! without.

use crate::Result;
/// Broad family of a fitting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub enum FittingCategory {
    Elbow,
    Reducer,
    Expander,
    Transition,
    Tee,
    Damper,
    Diffuser,
    Grille,
    Louver,
    Filter,
    Silencer,
    Entrance,
    Exit,
}

/// Which velocity the loss coefficient is referred to (ASHRAE convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub enum VelocityRef {
    /// Referred to the upstream (approach) velocity.
    Inlet,
    /// Referred to the downstream (leaving) velocity.
    Outlet,
    /// Referred to the main-line velocity (tees/crosses).
    Main,
    /// Referred to the branch-duct velocity.
    Branch,
}

/// One catalogued ζ value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub struct ZetaEntry {
    /// Stable lookup key, e.g. `"elbow.round.rd1.0"` or a vendor code.
    pub key: String,
    /// Human-readable name.
    pub name: String,
    pub category: FittingCategory,
    /// Loss coefficient ζ (dimensionless).
    pub zeta: f64,
    /// The velocity this coefficient refers to.
    pub reference_velocity: VelocityRef,
    /// Bibliographic / vendor source, e.g. `"SMACNA Duct Design"`,
    /// `"Idelchik §6"`, or a vendor catalogue name.
    pub source: String,
    /// Nominal duct size the constant applies to, in mm (None = size-averaged).
    pub size_mm: Option<f64>,
}

/// A mutable ζ database supporting catalogued lookup.
#[derive(Debug, Clone, Default)]
pub struct ZetaCatalog {
    entries: Vec<ZetaEntry>,
}

impl ZetaCatalog {
    pub fn new() -> Self {
        ZetaCatalog {
            entries: Vec::new(),
        }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add or replace an entry by key.
    pub fn insert(&mut self, entry: ZetaEntry) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.key == entry.key) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// Look up a catalogued ζ by key.
    pub fn get(&self, key: &str) -> Option<&ZetaEntry> {
        self.entries.iter().find(|e| e.key == key)
    }

    /// Convenience: the ζ value for a key, if present.
    pub fn lookup(&self, key: &str) -> Option<f64> {
        self.get(key).map(|e| e.zeta)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ZetaEntry> {
        self.entries.iter()
    }

    /// All entries whose category matches.
    pub fn by_category(&self, cat: FittingCategory) -> impl Iterator<Item = &ZetaEntry> {
        self.entries.iter().filter(move |e| e.category == cat)
    }

    /// All entries whose `source` contains `vendor_substring`, case-insensitively.
    ///
    /// Useful for filtering a merged catalogue down to one vendor's sheet
    /// (e.g. `by_vendor("Lindab")` or `by_vendor("alnor")`).
    pub fn by_vendor(&self, vendor_substring: &str) -> Vec<&ZetaEntry> {
        let needle = vendor_substring.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.source.to_lowercase().contains(&needle))
            .collect()
    }

    /// Merge in entries from another catalogue / vendor sheet.
    pub fn merge(&mut self, other: &ZetaCatalog) {
        for e in &other.entries {
            self.insert(e.clone());
        }
    }

    /// Load a catalogue from a slice of entries.
    pub fn from_entries(entries: Vec<ZetaEntry>) -> Self {
        ZetaCatalog { entries }
    }
}

/// Build a compact entry.
fn entry(
    key: &str,
    name: &str,
    category: FittingCategory,
    zeta: f64,
    reference_velocity: VelocityRef,
    source: &str,
    size_mm: Option<f64>,
) -> ZetaEntry {
    ZetaEntry {
        key: key.to_string(),
        name: name.to_string(),
        category,
        zeta,
        reference_velocity,
        source: source.to_string(),
        size_mm,
    }
}

/// The built-in reference catalogue — published constant ζ values for common
/// fittings, each tagged with its source and reference velocity.
///
/// # Examples
/// ```
/// use venti::reference_catalog;
/// let cat = reference_catalog();
/// assert_eq!(cat.lookup("elbow.round.rd1.5"), Some(0.21));
/// assert_eq!(cat.lookup("entrance.abrupt"), Some(0.50));
/// ```
pub fn reference_catalog() -> ZetaCatalog {
    use FittingCategory::*;
    use VelocityRef::*;
    ZetaCatalog::from_entries(vec![
        // ---- elbows ----
        entry(
            "elbow.round.rd1.5",
            "Round elbow 90°, R/D=1.5",
            Elbow,
            0.21,
            Outlet,
            "Idelchik §6 / ASHRAE Fund.",
            Some(200.0),
        ),
        entry(
            "elbow.round.rd1.0",
            "Round elbow 90°, R/D=1.0",
            Elbow,
            0.30,
            Outlet,
            "Idelchik §6",
            Some(200.0),
        ),
        entry(
            "elbow.rect.rw1.0",
            "Rectangular elbow 90°, R/W=1.0",
            Elbow,
            0.22,
            Outlet,
            "SMACNA Duct Design",
            Some(300.0),
        ),
        entry(
            "elbow.mitered.90",
            "Mitered elbow 90° (unvaned)",
            Elbow,
            1.20,
            Outlet,
            "ASHRAE Fund.",
            None,
        ),
        entry(
            "elbow.mitered.45",
            "Mitered elbow 45° (unvaned)",
            Elbow,
            0.44,
            Outlet,
            "ASHRAE Fund.",
            None,
        ),
        // ---- reducers / expanders / transitions ----
        entry(
            "reducer.smooth",
            "Smooth round reducer",
            Reducer,
            0.08,
            Outlet,
            "ASHRAE Fund. F29",
            None,
        ),
        entry(
            "expander.smooth",
            "Smooth round expander",
            Expander,
            0.15,
            Inlet,
            "ASHRAE Fund. F29",
            None,
        ),
        entry(
            "transition.equal",
            "Equal-area transition",
            Transition,
            0.04,
            Outlet,
            "SMACNA Duct Design",
            None,
        ),
        // ---- tees ----
        entry(
            "tee.branch.typical",
            "Tee branch leg (typical)",
            Tee,
            0.60,
            Branch,
            "ASHRAE Fund. F25",
            None,
        ),
        entry(
            "tee.straight.typical",
            "Tee straight-through leg",
            Tee,
            0.10,
            Main,
            "ASHRAE Fund. F25",
            None,
        ),
        // ---- dampers ----
        entry(
            "damper.butterfly.open",
            "Butterfly damper, fully open",
            Damper,
            0.10,
            Outlet,
            "ASHRAE / SMACNA",
            None,
        ),
        entry(
            "damper.fire.open",
            "Fire damper, open",
            Damper,
            0.18,
            Outlet,
            "ASHRAE / SMACNA",
            None,
        ),
        entry(
            "damper.volume.open",
            "Volume damper, fully open",
            Damper,
            0.10,
            Outlet,
            "SMACNA",
            None,
        ),
        // ---- vendor fire dampers (Trox / Mercor) ----
        // Representative fully-open housing section zetas at common nominal
        // round sizes, keyed `damper.fire.<brand>.<size_mm>` for the
        // `fire_damper_branded` selection helper in
        // `components::fittings_library`. Smaller dampers carry a slightly
        // higher open-housing loss (blade pack in a shorter collar).
        entry(
            "damper.fire.trox.160",
            "Fire damper, Trox FK-EU D 160 (open)",
            Damper,
            0.20,
            Outlet,
            "Trox catalogue",
            Some(160.0),
        ),
        entry(
            "damper.fire.trox.200",
            "Fire damper, Trox FK-EU D 200 (open)",
            Damper,
            0.20,
            Outlet,
            "Trox catalogue",
            Some(200.0),
        ),
        entry(
            "damper.fire.trox.250",
            "Fire damper, Trox FK-EU D 250 (open)",
            Damper,
            0.19,
            Outlet,
            "Trox catalogue",
            Some(250.0),
        ),
        entry(
            "damper.fire.trox.315",
            "Fire damper, Trox FK-EU D 315 (open)",
            Damper,
            0.18,
            Outlet,
            "Trox catalogue",
            Some(315.0),
        ),
        entry(
            "damper.fire.trox.400",
            "Fire damper, Trox FK-EU D 400 (open)",
            Damper,
            0.18,
            Outlet,
            "Trox catalogue",
            Some(400.0),
        ),
        entry(
            "damper.fire.trox.500",
            "Fire damper, Trox FK-EU D 500 (open)",
            Damper,
            0.18,
            Outlet,
            "Trox catalogue",
            Some(500.0),
        ),
        entry(
            "damper.fire.mercor.160",
            "Fire damper, Mercor MF D 160 (open)",
            Damper,
            0.21,
            Outlet,
            "Mercor catalogue",
            Some(160.0),
        ),
        entry(
            "damper.fire.mercor.200",
            "Fire damper, Mercor MF D 200 (open)",
            Damper,
            0.20,
            Outlet,
            "Mercor catalogue",
            Some(200.0),
        ),
        entry(
            "damper.fire.mercor.250",
            "Fire damper, Mercor MF D 250 (open)",
            Damper,
            0.19,
            Outlet,
            "Mercor catalogue",
            Some(250.0),
        ),
        entry(
            "damper.fire.mercor.315",
            "Fire damper, Mercor MF D 315 (open)",
            Damper,
            0.19,
            Outlet,
            "Mercor catalogue",
            Some(315.0),
        ),
        entry(
            "damper.fire.mercor.400",
            "Fire damper, Mercor MF D 400 (open)",
            Damper,
            0.18,
            Outlet,
            "Mercor catalogue",
            Some(400.0),
        ),
        entry(
            "damper.fire.mercor.500",
            "Fire damper, Mercor MF D 500 (open)",
            Damper,
            0.18,
            Outlet,
            "Mercor catalogue",
            Some(500.0),
        ),
        // ---- diffusers / grilles ----
        entry(
            "diffuser.ceiling",
            "Ceiling diffuser, face",
            Diffuser,
            0.40,
            Outlet,
            "ASHRAE Fund. F25",
            None,
        ),
        entry(
            "diffuser.slot",
            "Linear slot diffuser",
            Diffuser,
            0.30,
            Outlet,
            "Manufacturer data",
            None,
        ),
        entry(
            "grille.return",
            "Return grille",
            Grille,
            0.25,
            Inlet,
            "ASHRAE Fund. F25",
            None,
        ),
        // ---- louvers / filters / silencers ----
        entry(
            "louver.open",
            "Weather louver, open",
            Louver,
            0.25,
            Inlet,
            "ASHRAE Fund. F25",
            None,
        ),
        entry(
            "filter.panel.open",
            "Panel filter bank (clean)",
            Filter,
            0.12,
            Inlet,
            "Filter mfr. data",
            None,
        ),
        entry(
            "silencer.open",
            "Duct silencer (open)",
            Silencer,
            0.35,
            Outlet,
            "Silencer mfr. data",
            None,
        ),
        // ---- entrances / exits ----
        entry(
            "entrance.abrupt",
            "Abrupt duct entrance",
            Entrance,
            0.50,
            Inlet,
            "Idelchik §4",
            None,
        ),
        entry(
            "entrance.rounded",
            "Rounded duct entrance",
            Entrance,
            0.03,
            Inlet,
            "Idelchik §4",
            None,
        ),
        entry(
            "exit.abrupt",
            "Abrupt duct exit",
            Exit,
            1.00,
            Inlet,
            "Borda–Carnot / Idelchik",
            None,
        ),
        entry(
            "exit.discharge",
            "Duct discharge to room",
            Exit,
            1.00,
            Inlet,
            "ASHRAE Fund.",
            None,
        ),
        // ---- Lindab / Alnor placeholders (so by_vendor works on the built-in) ----
        entry(
            "lindab.elbow.round.rd1.0",
            "Lindab round elbow 90°, R/D = 1.0",
            Elbow,
            0.30,
            Outlet,
            "Lindab",
            Some(200.0),
        ),
        entry(
            "lindab.damper.fire.200",
            "Lindab fire damper, open, D 200",
            Damper,
            0.20,
            Outlet,
            "Lindab",
            Some(200.0),
        ),
        entry(
            "alnor.grille.return",
            "Alnor return grille",
            Grille,
            0.31,
            Inlet,
            "Alnor",
            None,
        ),
    ])
}

// ---------------------------------------------------------------------------
// Vendor catalogue (serde, behind the `cli` feature)
// ---------------------------------------------------------------------------

/// A vendor catalogue sheet: a named set of [`ZetaEntry`]s, loadable from JSON.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct VendorCatalog {
    pub vendor: String,
    pub fittings: Vec<ZetaEntry>,
}

#[cfg(feature = "cli")]
impl VendorCatalog {
    /// Convert to a [`ZetaCatalog`] keyed by each entry's `key`.
    pub fn to_catalog(&self) -> ZetaCatalog {
        ZetaCatalog::from_entries(self.fittings.clone())
    }
}

/// Parse a vendor catalogue from a JSON string.
#[cfg(feature = "cli")]
pub fn vendor_catalog_from_json(json: &str) -> Result<VendorCatalog> {
    Ok(serde_json::from_str(json).map_err(|e| format!("catalogue JSON: {e}"))?)
}

/// Parse a vendor catalogue from a JSON string and return the entries as a
/// [`ZetaCatalog`]. Thin convenience wrapper over [`vendor_catalog_from_json`].
#[cfg(feature = "cli")]
pub fn from_vendor_json(json: &str) -> Result<ZetaCatalog> {
    Ok(vendor_catalog_from_json(json)?.to_catalog())
}

/// Load a vendor catalogue from a `.json` file.
#[cfg(feature = "cli")]
pub fn vendor_catalog_from_file(path: &str) -> Result<VendorCatalog> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {path:?}: {e}"))?;
    vendor_catalog_from_json(&text)
}

#[cfg(all(test, feature = "cli"))]
mod tests {
    use super::*;

    #[test]
    fn fire_damper_vendor_entries_exist() {
        let cat = reference_catalog();
        let cases: &[(&str, &str, &[f64])] = &[
            (
                "trox",
                "Trox catalogue",
                &[160.0, 200.0, 250.0, 315.0, 400.0, 500.0],
            ),
            (
                "mercor",
                "Mercor catalogue",
                &[160.0, 200.0, 250.0, 315.0, 400.0, 500.0],
            ),
        ];
        for (brand, source, sizes) in cases {
            for s in *sizes {
                let key = format!("damper.fire.{brand}.{s}");
                let e = cat.get(&key).unwrap_or_else(|| panic!("missing {key}"));
                assert_eq!(e.category, FittingCategory::Damper);
                assert_eq!(e.reference_velocity, VelocityRef::Outlet);
                assert_eq!(e.source, *source);
                assert_eq!(e.size_mm, Some(*s));
                assert!(
                    (0.18..=0.22).contains(&e.zeta),
                    "{key} zeta {} outside [0.18, 0.22]",
                    e.zeta
                );
            }
        }
        assert!(cat.by_category(FittingCategory::Damper).count() >= 12);
    }

    #[test]
    fn reference_catalog_lookup() {
        let cat = reference_catalog();
        assert!(cat.len() >= 20);
        assert!((cat.lookup("elbow.round.rd1.0").unwrap() - 0.30).abs() < 1e-9);
        assert_eq!(cat.lookup("no_such_key"), None);
        let elbows: Vec<_> = cat.by_category(FittingCategory::Elbow).collect();
        assert!(elbows.len() >= 5);
    }

    #[test]
    fn reference_entries_are_documented() {
        let cat = reference_catalog();
        for e in cat.iter() {
            assert!(!e.source.is_empty(), "entry {} needs a source", e.key);
            assert!(e.zeta > 0.0);
        }
    }

    #[test]
    fn vendor_json_round_trip() {
        let json = r#"{
            "vendor": "Example HVAC",
            "fittings": [
                {"key":"elbow.round.rd1.0","name":"90 round elbow R/D1","category":"Elbow",
                 "zeta":0.29,"reference_velocity":"Outlet","source":"vendor cat","size_mm":200},
                {"key":"grille.x","name":"X grille","category":"Grille",
                 "zeta":0.31,"reference_velocity":"Inlet","source":"vendor cat","size_mm":null}
            ]
        }"#;
        let vc = vendor_catalog_from_json(json).unwrap();
        assert_eq!(vc.vendor, "Example HVAC");
        assert_eq!(vc.fittings.len(), 2);
        let cat = vc.to_catalog();
        assert!((cat.lookup("elbow.round.rd1.0").unwrap() - 0.29).abs() < 1e-9);

        // round-trip back to JSON serializes to a valid object
        let out = serde_json::to_string(&vc).unwrap();
        let back: VendorCatalog = serde_json::from_str(&out).unwrap();
        assert_eq!(back.fittings.len(), 2);
    }

    #[test]
    fn merge_vendor_overwrites_reference() {
        let mut cat = reference_catalog();
        let vc = vendor_catalog_from_json(
            r#"{"vendor":"V","fittings":[
              {"key":"elbow.round.rd1.0","name":"vendor elbow","category":"Elbow",
               "zeta":0.25,"reference_velocity":"Outlet","source":"V","size_mm":null}]}"#,
        )
        .unwrap();
        cat.merge(&vc.to_catalog());
        assert!((cat.lookup("elbow.round.rd1.0").unwrap() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn by_vendor_filters_case_insensitive() {
        let cat = reference_catalog();
        // Case-insensitive on a built-in placeholder.
        assert!(!cat.by_vendor("LINdab").is_empty());
        assert!(!cat.by_vendor("lindab").is_empty());
        assert!(!cat.by_vendor("alnor").is_empty());
        // A source not present.
        assert!(cat.by_vendor("nobody-here").is_empty());
    }

    #[test]
    fn from_vendor_json_parses_sheet() {
        let json = r#"{
            "vendor": "Lindab-Alnor",
            "fittings": [
                {"key":"lindab.elbow.round.rd1.0","name":"e","category":"Elbow",
                 "zeta":0.30,"reference_velocity":"Outlet","source":"Lindab","size_mm":200},
                {"key":"alnor.grille.return","name":"g","category":"Grille",
                 "zeta":0.31,"reference_velocity":"Inlet","source":"Alnor","size_mm":null}
            ]
        }"#;
        let cat = from_vendor_json(json).unwrap();
        assert_eq!(cat.len(), 2);
        assert_eq!(cat.by_vendor("Lindab").len(), 1);
        assert_eq!(cat.by_vendor("Alnor").len(), 1);
        assert!((cat.lookup("lindab.elbow.round.rd1.0").unwrap() - 0.30).abs() < 1e-9);
    }

    #[test]
    fn lindab_alnor_example_sheet_loads() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("vendor_lindab_alnor.json");
        let json = std::fs::read_to_string(&path).unwrap();
        let cat = from_vendor_json(&json).unwrap();
        assert!(cat.len() >= 8, "expected >=8 entries, got {}", cat.len());
        assert!(!cat.by_vendor("Lindab").is_empty());
        assert!(!cat.by_vendor("Alnor").is_empty());
    }

    #[test]
    fn merge_lindab_alnor_into_reference() {
        // Merged lookup: vendor sheet overrides / adds, then by_vendor slices it.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("vendor_lindab_alnor.json");
        let json = std::fs::read_to_string(&path).unwrap();
        let vendor = from_vendor_json(&json).unwrap();
        let mut cat = reference_catalog();
        cat.merge(&vendor);
        // Vendor entry visible through merged catalogue.
        assert!(!cat.by_vendor("Lindab").is_empty());
        assert!(!cat.by_vendor("alnor").is_empty());
        // Distinct vendor buckets don't cross-contaminate by key.
        let lindab_keys: Vec<_> = cat.by_vendor("Lindab").iter().map(|e| &e.key).collect();
        assert!(lindab_keys.iter().all(|k| k.starts_with("lindab.")));
        let alnor_keys: Vec<_> = cat.by_vendor("Alnor").iter().map(|e| &e.key).collect();
        assert!(alnor_keys.iter().all(|k| k.starts_with("alnor.")));
    }
}

#[cfg(all(test, not(feature = "cli")))]
mod tests_nocli {
    use super::*;
    // Core always builds; lookup must work without serde too.
    #[test]
    fn fire_damper_vendor_entries_core() {
        // The vendor fire-damper data must exist in the core (non-serde) build too.
        let cat = reference_catalog();
        let e = cat.get("damper.fire.trox.200").unwrap();
        assert_eq!(e.source, "Trox catalogue");
        assert_eq!(e.category, FittingCategory::Damper);
        assert_eq!(e.reference_velocity, VelocityRef::Outlet);
        let m = cat.get("damper.fire.mercor.315").unwrap();
        assert_eq!(m.source, "Mercor catalogue");
        assert_eq!(m.size_mm, Some(315.0));
    }

    #[test]
    fn reference_catalog_core_lookup() {
        let cat = reference_catalog();
        assert!((cat.lookup("entrance.abrupt").unwrap() - 0.50).abs() < 1e-9);
        assert!(cat.get("tee.branch.typical").unwrap().reference_velocity == VelocityRef::Branch);
    }
}
