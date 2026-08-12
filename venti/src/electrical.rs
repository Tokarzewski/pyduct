//! Electrical equipment data model and schedule — "zestawienie danych
//! elektrycznych".
//!
//! A small, dependency-free model for recording the nameplate electrical data
//! of equipment (fans, pumps, AHUs, heaters, …) and for rendering it as a
//! tabular CSV schedule suitable for a documentation handover.
//!
//! * [`ElectricalData`] captures the derived, optional values such as supply
//!   voltage, operating current and power factor.
//! * [`ElectricalSchedule`] groups entries and computes totals
//!   (installed power, summed operating current).
//! * [`electrical_as_csv`] renders the schedule as a flat CSV report.

use crate::Result;

/// Nameplate electrical data for a single equipment item.
pub struct ElectricalData {
    /// Identifier of the component in the wider model (e.g. `"F-01"`).
    pub component_id: String,
    /// Equipment type (e.g. `"supply fan"`, `"circulation pump"`).
    pub device_type: String,
    /// Installed electrical power in watts.
    pub power_w: f64,
    /// Supply voltage in volts, when known.
    pub voltage_v: Option<f64>,
    /// Operating current in amperes, when known or computable.
    pub current_a: Option<f64>,
    /// Power factor (cos φ), when known.
    pub power_factor: Option<f64>,
    /// Supply frequency in hertz, when known.
    pub frequency_hz: Option<f64>,
}

impl ElectricalData {
    /// Create a new electrical-data record with the given installed power.
    ///
    /// Returns an error when `power_w` is negative or not a finite number.
    pub fn new(
        component_id: impl Into<String>,
        device_type: impl Into<String>,
        power_w: f64,
    ) -> Result<Self> {
        if !power_w.is_finite() || power_w < 0.0 {
            return Err("power_w must be a non-negative finite value".into());
        }
        Ok(Self {
            component_id: component_id.into(),
            device_type: device_type.into(),
            power_w,
            voltage_v: None,
            current_a: None,
            power_factor: None,
            frequency_hz: None,
        })
    }

    /// Compute the operating current `I = P / (U · cos φ)` from the installed
    /// power, voltage and power factor, and store it in [`Self::current_a`].
    ///
    /// Returns `None` (and clears `current_a`) when either the voltage or the
    /// power factor is missing or non-positive.
    ///
    /// # Examples
    ///
    /// ```
    /// use venti::ElectricalData;
    ///
    /// let mut pump = ElectricalData::new("P-01", "circulation pump", 1_500.0).unwrap();
    /// pump.voltage_v = Some(230.0);
    /// pump.power_factor = Some(0.9);
    ///
    /// let amps = pump.current().unwrap();
    /// assert!((amps - 1500.0 / (230.0 * 0.9)).abs() < 1e-9);
    /// ```
    pub fn current(&mut self) -> Option<f64> {
        let amps = self.computed_current();
        self.current_a = amps;
        amps
    }

    /// Installed power expressed in kilowatts.
    pub fn power_kw(&self) -> f64 {
        self.power_w / 1000.0
    }

    /// Current in amperes derived from voltage and power factor alone
    /// (does not touch the stored [`Self::current_a`] field).
    fn computed_current(&self) -> Option<f64> {
        match (self.voltage_v, self.power_factor) {
            (Some(v), Some(pf)) if v > 0.0 && pf > 0.0 => Some(self.power_w / (v * pf)),
            _ => None,
        }
    }
}

/// A tabular schedule ("zestawienie") of electrical equipment records.
pub struct ElectricalSchedule {
    entries: Vec<ElectricalData>,
}

impl ElectricalSchedule {
    /// Create an empty schedule.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append one electrical-data record to the schedule.
    pub fn add(&mut self, entry: ElectricalData) {
        self.entries.push(entry);
    }

    /// Total installed electrical power across all entries, in watts.
    ///
    /// # Examples
    ///
    /// ```
    /// use venti::{ElectricalData, ElectricalSchedule};
    ///
    /// let mut schedule = ElectricalSchedule::new();
    /// schedule.add(ElectricalData::new("F-01", "supply fan", 5_500.0).unwrap());
    /// schedule.add(ElectricalData::new("P-01", "circulation pump", 1_500.0).unwrap());
    ///
    /// assert_eq!(schedule.total_power_w(), 7_000.0);
    /// ```
    pub fn total_power_w(&self) -> f64 {
        self.entries.iter().map(|e| e.power_w).sum()
    }

    /// Sum of the per-entry operating currents (amperes).
    ///
    /// Returns `None` as soon as any entry is missing the voltage or power
    /// factor needed to derive its current.
    pub fn total_current_a(&self) -> Option<f64> {
        let mut total = 0.0;
        for e in &self.entries {
            total += e.computed_current()?;
        }
        Some(total)
    }

    /// Number of records in the schedule.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the schedule contains no records.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over the schedule's records.
    pub fn iter(&self) -> impl Iterator<Item = &ElectricalData> {
        self.entries.iter()
    }
}

impl Default for ElectricalSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Render an [`ElectricalSchedule`] as a CSV report (the "zestawienie danych
/// elektrycznych" handover sheet).
///
/// The header line is
/// `component_id,device_type,power_w,power_kw,voltage_v,current_a,power_factor,frequency_hz`.
/// Missing optional values (voltage, current, power factor, frequency) are
/// rendered as empty fields; the `current_a` column shows the stored value
/// when present and otherwise falls back to the value computable from
/// power / (voltage · power factor).
pub fn electrical_as_csv(schedule: &ElectricalSchedule) -> String {
    let mut out = String::from(
        "component_id,device_type,power_w,power_kw,voltage_v,current_a,power_factor,frequency_hz",
    );
    for e in &schedule.entries {
        let current = match e.current_a {
            Some(i) => i.to_string(),
            None => e
                .computed_current()
                .map(|i| i.to_string())
                .unwrap_or_default(),
        };
        let component_id = &e.component_id;
        let device_type = &e.device_type;
        let power_w = e.power_w.to_string();
        let power_kw = e.power_kw().to_string();
        let voltage = e.voltage_v.map(|v| v.to_string()).unwrap_or_default();
        let power_factor = e.power_factor.map(|v| v.to_string()).unwrap_or_default();
        let frequency = e.frequency_hz.map(|v| v.to_string()).unwrap_or_default();
        out.push('\n');
        out.push_str(&format!(
            "{component_id},{device_type},{power_w},{power_kw},{voltage},{current},{power_factor},{frequency}"
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pump() -> ElectricalData {
        ElectricalData::new("P-01", "centrifugal pump", 1_500.0).unwrap()
    }

    #[test]
    fn current_computation_from_voltage_and_power_factor() {
        let mut p = pump();
        p.voltage_v = Some(230.0);
        p.power_factor = Some(0.9);
        let expected = 1500.0 / (230.0 * 0.9);
        let amps = p.current().unwrap();
        assert!((amps - expected).abs() < 1e-9);
        // also stored back into the record
        assert!((p.current_a.unwrap() - expected).abs() < 1e-9);
    }

    #[test]
    fn power_kw_conversion() {
        let p = pump();
        assert!((p.power_kw() - 1.5).abs() < 1e-12);
        let big = ElectricalData::new("H-01", "heater", 12_000.0).unwrap();
        assert!((big.power_kw() - 12.0).abs() < 1e-12);
    }

    #[test]
    fn schedule_totals_power() {
        let mut s = ElectricalSchedule::new();
        s.add(ElectricalData::new("F-01", "supply fan", 5_500.0).unwrap());
        s.add(ElectricalData::new("P-01", "circulation pump", 1_500.0).unwrap());
        s.add(ElectricalData::new("H-01", "electric heater", 2_000.0).unwrap());
        assert!((s.total_power_w() - 9_000.0).abs() < 1e-9);
    }

    #[test]
    fn schedule_totals_current_when_all_present() {
        let mut s = ElectricalSchedule::new();
        let mut fan = ElectricalData::new("F-01", "supply fan", 5_500.0).unwrap();
        fan.voltage_v = Some(400.0);
        fan.power_factor = Some(0.85);
        s.add(fan);
        let mut pump = ElectricalData::new("P-01", "circulation pump", 1_500.0).unwrap();
        pump.voltage_v = Some(230.0);
        pump.power_factor = Some(0.9);
        s.add(pump);
        let expected = 5500.0 / (400.0 * 0.85) + 1500.0 / (230.0 * 0.9);
        assert!((s.total_current_a().unwrap() - expected).abs() < 1e-9);
    }

    #[test]
    fn missing_voltage_or_power_factor_gives_none() {
        // no voltage, no power factor
        let mut p = pump();
        assert_eq!(p.current(), None);
        assert_eq!(p.current_a, None);
        // power factor alone is not enough
        p.power_factor = Some(0.9);
        assert_eq!(p.current(), None);
        // voltage alone is not enough
        let mut m = ElectricalData::new("M-01", "motor", 1_000.0).unwrap();
        m.voltage_v = Some(230.0);
        assert_eq!(m.current(), None);
    }

    #[test]
    fn total_current_is_none_when_any_entry_is_missing_data() {
        let mut s = ElectricalSchedule::new();
        let mut fan = ElectricalData::new("F-01", "supply fan", 5_500.0).unwrap();
        fan.voltage_v = Some(400.0);
        fan.power_factor = Some(0.85);
        s.add(fan);
        // entry without voltage/power factor
        s.add(ElectricalData::new("H-01", "electric heater", 2_000.0).unwrap());
        assert_eq!(s.total_current_a(), None);
    }

    #[test]
    fn csv_header_and_rows() {
        let mut fan = ElectricalData::new("F-01", "supply fan", 5_500.0).unwrap();
        fan.voltage_v = Some(400.0);
        fan.power_factor = Some(0.85);
        fan.frequency_hz = Some(50.0);
        fan.current().unwrap();
        let mut s = ElectricalSchedule::new();
        s.add(fan);
        s.add(ElectricalData::new("H-01", "electric heater", 2_000.0).unwrap());

        let csv = electrical_as_csv(&s);
        let mut lines = csv.lines();
        assert_eq!(
            lines.next().unwrap(),
            "component_id,device_type,power_w,power_kw,voltage_v,current_a,power_factor,frequency_hz"
        );
        let row = lines.next().unwrap();
        assert!(row.starts_with("F-01,supply fan,5500,5.5,400,"));
        assert!(row.ends_with(",0.85,50"));
        assert_eq!(lines.next().unwrap(), "H-01,electric heater,2000,2,,,,");
        assert!(lines.next().is_none());
    }

    #[test]
    fn rejects_negative_and_non_finite_power() {
        assert!(ElectricalData::new("X-01", "heater", -5.0).is_err());
        assert!(ElectricalData::new("X-02", "heater", f64::NAN).is_err());
        assert!(ElectricalData::new("X-03", "heater", f64::INFINITY).is_err());
        assert!(ElectricalData::new("X-04", "heater", 0.0).is_ok());
    }

    #[test]
    fn add_and_len() {
        let mut s = ElectricalSchedule::new();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
        s.add(ElectricalData::new("A-01", "AHU", 2_000.0).unwrap());
        s.add(ElectricalData::new("A-02", "AHU", 3_000.0).unwrap());
        assert_eq!(s.len(), 2);
        assert!(!s.is_empty());
        assert_eq!(s.iter().count(), 2);
    }
}
