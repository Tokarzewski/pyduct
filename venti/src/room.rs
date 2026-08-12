//! Room ventilation air balance — per-room supply vs. exhaust, imbalance and
//! air-change rate (bilans powietrza w pomieszczeniach).
//!
//! HVAC design requires every room's **supply** (nawiew) and **exhaust**
//! (wywiew) flows to be tracked and reconciled: bathrooms/kitchens are net
//! exhaust, cleanrooms/corridors are net supply, and mechanical rooms typically
//! target near-zero net flow. This module provides:
//!
//! 1. [`RoomBalance`] — one room's supply/exhaust pair plus the derived
//!    net flow and imbalance metrics (absolute `m³/s` and dimensionless
//!    fraction of the larger flow).
//! 2. [`room_ach`] — air changes per hour from a flow and room volume
//!    (re-exporting the core [`air_changes_per_hour`](crate::units::air_changes_per_hour)).
//! 3. [`RoomBalanceSet`] — a named collection of room balances with totals,
//!    overall balance check and CSV rendering (optionally including ACH when
//!    room volumes are known).
//!
//! All quantities are SI: flows in `m³/s`, volumes in `m³`.

use std::collections::HashMap;

use crate::units::air_changes_per_hour;
use crate::Result;

/// Supply/exhaust flow pair for a single room, in `m³/s`.
///
/// `supply_m3s` is the supply airflow into the room (nawiew), `exhaust_m3s`
/// the airflow extracted from it (wywiew). A room whose supply exceeds exhaust
/// is pressurised (positive net flow); the reverse is depressurised.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomBalance {
    /// Supply airflow into the room, `m³/s` (≥ 0).
    pub supply_m3s: f64,
    /// Exhaust airflow out of the room, `m³/s` (≥ 0).
    pub exhaust_m3s: f64,
}

impl RoomBalance {
    /// Build a room balance, rejecting negative flows.
    ///
    /// # Errors
    ///
    /// Returns `Err` if either flow is negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use venti::RoomBalance;
    ///
    /// let bal = RoomBalance::new(0.15, 0.10).unwrap();
    /// assert_eq!(bal.supply_m3s, 0.15);
    /// assert_eq!(bal.exhaust_m3s, 0.10);
    /// assert!(!bal.is_balanced(0.0));
    ///
    /// // Negative flows are rejected.
    /// assert!(RoomBalance::new(-0.1, 0.2).is_err());
    /// ```
    pub fn new(supply_m3s: f64, exhaust_m3s: f64) -> Result<Self> {
        if supply_m3s < 0.0 {
            return Err("supply_m3s must be non-negative".into());
        }
        if exhaust_m3s < 0.0 {
            return Err("exhaust_m3s must be non-negative".into());
        }
        Ok(RoomBalance {
            supply_m3s,
            exhaust_m3s,
        })
    }

    /// Net room flow, `m³/s` — positive means excess supply (nawiew)
    /// pressurising the room, negative means excess exhaust.
    #[inline]
    pub fn net_m3s(&self) -> f64 {
        self.supply_m3s - self.exhaust_m3s
    }

    /// Whether the room is within `tolerance` (in `m³/s`) of a perfect
    /// supply/exhaust match, i.e. `|net| ≤ tolerance`.
    ///
    /// A negative tolerance is treated as unsatisfiable (never balanced).
    #[inline]
    pub fn is_balanced(&self, tolerance: f64) -> bool {
        tolerance >= 0.0 && self.net_m3s().abs() <= tolerance
    }

    /// Dimensionless imbalance: `net / max(supply, exhaust)`, in `[-1, 1]`.
    ///
    /// `+1` means pure supply (no exhaust), `-1` pure exhaust (no supply),
    /// `0` perfectly balanced. Returns `None` when both flows are zero — the
    /// ratio is undefined (the room is trivially balanced but carries no air).
    pub fn imbalance_fraction(&self) -> Option<f64> {
        let peak = self.supply_m3s.max(self.exhaust_m3s);
        if peak <= 0.0 {
            None
        } else {
            Some(self.net_m3s() / peak)
        }
    }
}

/// Air changes per hour (ACH) for a room: how many times the room volume is
/// exchanged in an hour at the given flow.
///
/// Delegates to [`air_changes_per_hour`](crate::units::air_changes_per_hour);
/// `volume_m3` must be positive and `flow_m3s` non-negative.
///
/// # Examples
///
/// ```
/// use venti::room_ach;
///
/// // 0.1 m³/s through a 100 m³ room = 3.6 air changes per hour.
/// let ach = room_ach(0.1, 100.0).unwrap();
/// assert!((ach - 3.6).abs() < 1e-9);
///
/// // Zero volume is rejected.
/// assert!(room_ach(0.1, 0.0).is_err());
/// ```
#[inline]
pub fn room_ach(flow_m3s: f64, volume_m3: f64) -> Result<f64> {
    air_changes_per_hour(flow_m3s, volume_m3)
}

/// A named collection of room balances, with totals, overall balance and
/// CSV rendering.
///
/// Rooms are stored in insertion order; totals and the overall balance are
/// computed across all of them. An optional room volume can be recorded via
/// [`add_with_volume`](Self::add_with_volume) so that [`csv_render`](Self::csv_render)
/// can fill the `ach` column; rooms added with plain [`add`](Self::add) have no
/// known volume and render an empty ACH field.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RoomBalanceSet {
    rooms: Vec<(String, RoomBalance)>,
    volumes: HashMap<String, f64>,
}

impl RoomBalanceSet {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a room (without a known volume — the `ach` CSV column is left
    /// empty for it). Use [`add_with_volume`](Self::add_with_volume) when the
    /// room volume is available.
    pub fn add(&mut self, name: impl Into<String>, bal: RoomBalance) {
        self.rooms.push((name.into(), bal));
    }

    /// Append a room together with its volume (in `m³`) so its ACH can be
    /// computed. `volume_m3` must be positive; otherwise the room is stored
    /// with no volume and the `ach` column renders empty.
    pub fn add_with_volume(&mut self, name: impl Into<String>, bal: RoomBalance, volume_m3: f64) {
        let name = name.into();
        if volume_m3 > 0.0 {
            self.volumes.insert(name.clone(), volume_m3);
        }
        self.rooms.push((name, bal));
    }

    /// Total supply airflow across all rooms, `m³/s`.
    pub fn total_supply_m3s(&self) -> f64 {
        self.rooms.iter().map(|(_, b)| b.supply_m3s).sum()
    }

    /// Total exhaust airflow across all rooms, `m³/s`.
    pub fn total_exhaust_m3s(&self) -> f64 {
        self.rooms.iter().map(|(_, b)| b.exhaust_m3s).sum()
    }

    /// Overall net flow across all rooms, `m³/s` (sum of the per-room nets).
    #[inline]
    pub fn overall_net_m3s(&self) -> f64 {
        self.total_supply_m3s() - self.total_exhaust_m3s()
    }

    /// Whether the whole set is balanced to within `tolerance` `m³/s`, i.e.
    /// `|overall net| ≤ tolerance`.
    #[inline]
    pub fn is_balanced(&self, tolerance: f64) -> bool {
        tolerance >= 0.0 && self.overall_net_m3s().abs() <= tolerance
    }

    /// Render the set as CSV with header `name,supply_m3s,exhaust_m3s,net_m3s,ach`.
    ///
    /// The `ach` column holds `supply_m3s` converted to air changes per hour
    /// via [`room_ach`] for rooms whose volume was recorded with
    /// [`add_with_volume`](Self::add_with_volume), and is empty otherwise.
    /// Numbers are formatted with up to 6 decimal places (trailing zeros
    /// trimmed).
    pub fn csv_render(&self) -> String {
        let mut lines = vec!["name,supply_m3s,exhaust_m3s,net_m3s,ach".to_string()];
        for (name, bal) in &self.rooms {
            let ach = self
                .volumes
                .get(name)
                .and_then(|&v| room_ach(bal.supply_m3s, v).ok())
                .map(fmt_num)
                .unwrap_or_default();
            lines.push(format!(
                "{},{},{},{},{}",
                name,
                fmt_num(bal.supply_m3s),
                fmt_num(bal.exhaust_m3s),
                fmt_num(bal.net_m3s()),
                ach
            ));
        }
        lines.join("\n")
    }
}

/// Format a float with up to 6 decimal places, trimming trailing zeros —
/// `3.6000000000000005` → `"3.6"`.
fn fmt_num(v: f64) -> String {
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_net_sign() {
        // Excess supply (nawiew) → positive net; excess exhaust → negative.
        let over = RoomBalance::new(0.3, 0.1).unwrap();
        let under = RoomBalance::new(0.1, 0.3).unwrap();
        let equal = RoomBalance::new(0.2, 0.2).unwrap();
        assert!(over.net_m3s() > 0.0);
        assert!((over.net_m3s() - 0.2).abs() < 1e-12);
        assert!(under.net_m3s() < 0.0);
        assert!((under.net_m3s() + 0.2).abs() < 1e-12);
        assert_eq!(equal.net_m3s(), 0.0);
    }

    #[test]
    fn is_balanced_tolerance() {
        // 1.0 − 1.0625 = −0.0625 exactly, so comparison vs 0.0625 is exact.
        let bal = RoomBalance::new(1.0, 1.0625).unwrap();
        assert!(bal.is_balanced(0.0625)); // |net| == tolerance → balanced
        assert!(bal.is_balanced(0.1));
        assert!(!bal.is_balanced(0.0624));
        assert!(!bal.is_balanced(0.0));
    }

    #[test]
    fn is_balanced_monotonic_in_tolerance() {
        // net = 0.5 − 0.25 = 0.25 exactly.
        let bal = RoomBalance::new(0.5, 0.25).unwrap();
        let tols = [0.0, 0.1, 0.2, 0.249, 0.25, 0.251, 1.0];
        let results: Vec<bool> = tols.iter().map(|&t| bal.is_balanced(t)).collect();
        // Once balanced, stays balanced for larger tolerances.
        assert!(!results[0]);
        assert!(!results[3]);
        assert!(results[4]);
        for w in results.windows(2) {
            assert!(w[1] >= w[0], "is_balanced must be monotonic in tolerance");
        }
    }

    #[test]
    fn ach_value() {
        // 0.1 m³/s in a 100 m³ room = 3.6 ACH; monotonic in flow.
        assert!((room_ach(0.1, 100.0).unwrap() - 3.6).abs() < 1e-9);
        assert!((room_ach(0.2, 100.0).unwrap() - 7.2).abs() < 1e-9);
        assert!(room_ach(0.1, 50.0).unwrap() > room_ach(0.1, 100.0).unwrap());
        assert_eq!(room_ach(0.0, 100.0).unwrap(), 0.0);
    }

    #[test]
    fn validation_errors() {
        // RoomBalance rejects negative flows; room_ach rejects bad volume/flow.
        assert!(RoomBalance::new(-0.1, 0.2).is_err());
        assert!(RoomBalance::new(0.1, -0.2).is_err());
        assert!(RoomBalance::new(-0.1, -0.2).is_err());
        assert!(RoomBalance::new(0.0, 0.0).is_ok());

        assert!(room_ach(0.1, 0.0).is_err());
        assert!(room_ach(0.1, -50.0).is_err());
        assert!(room_ach(-1.0, 100.0).is_err());
    }

    #[test]
    fn imbalance_fraction_values() {
        // Pure supply → +1; pure exhaust → −1; balanced → 0; both zero → None.
        assert_eq!(
            RoomBalance::new(0.3, 0.0).unwrap().imbalance_fraction(),
            Some(1.0)
        );
        assert_eq!(
            RoomBalance::new(0.0, 0.3).unwrap().imbalance_fraction(),
            Some(-1.0)
        );
        assert_eq!(
            RoomBalance::new(0.2, 0.2).unwrap().imbalance_fraction(),
            Some(0.0)
        );
        // 0.5/0.25 → net 0.25, peak 0.5 → 0.25/0.5 = 0.5 exactly.
        assert_eq!(
            RoomBalance::new(0.5, 0.25).unwrap().imbalance_fraction(),
            Some(0.5)
        );
        assert_eq!(
            RoomBalance::new(0.25, 0.5).unwrap().imbalance_fraction(),
            Some(-0.5)
        );
        assert_eq!(
            RoomBalance::new(0.0, 0.0).unwrap().imbalance_fraction(),
            None
        );
    }

    #[test]
    fn totals_across_rooms() {
        // All flows are exact binary fractions, so the sum/net comparisons are exact.
        let mut set = RoomBalanceSet::new();
        set.add("bathroom", RoomBalance::new(0.0625, 0.125).unwrap()); // net −0.0625
        set.add("living", RoomBalance::new(0.25, 0.125).unwrap()); // net +0.125
        set.add("kitchen", RoomBalance::new(0.0625, 0.1875).unwrap()); // net −0.125
        assert_eq!(set.total_supply_m3s(), 0.375);
        assert_eq!(set.total_exhaust_m3s(), 0.4375);
        assert_eq!(set.overall_net_m3s(), -0.0625);
        assert!(set.is_balanced(0.0625)); // |net| == tol
        assert!(!set.is_balanced(0.0624));
    }

    #[test]
    fn csv_content() {
        let mut set = RoomBalanceSet::new();
        set.add_with_volume("bathroom", RoomBalance::new(0.02, 0.05).unwrap(), 5.0);
        set.add("corridor", RoomBalance::new(0.10, 0.10).unwrap());
        let csv = set.csv_render();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "name,supply_m3s,exhaust_m3s,net_m3s,ach");
        assert_eq!(lines[1], "bathroom,0.02,0.05,-0.03,14.4"); // 0.02*3600/5
                                                               // No volume recorded → ach column empty.
        assert_eq!(lines[2], "corridor,0.1,0.1,0,");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn csv_ach_via_add_with_volume() {
        // ACH column tracks supply flow only when a volume is present.
        let mut set = RoomBalanceSet::new();
        set.add_with_volume("office", RoomBalance::new(0.1, 0.1).unwrap(), 100.0);
        assert!(set.csv_render().contains(",3.6"));
    }
}
