//! Report export engine: Excel (`.xlsx`).
//!
//! This module turns in-memory report tables and the [`crate::electrical`]
//! schedule into binary Excel workbooks. It is the first half of issue #45
//! (ModelTok/pyduct); **PDF export is a documented follow-up** and will be
//! added here in a later round.
//!
//! The module is gated behind the `export` cargo feature so that the crate
//! core (`cargo build --no-default-features --lib`) stays dependency-free:
//! the only dependency it introduces, `rust_xlsxwriter`, is optional and only
//! pulled in when the feature is enabled.

use crate::Result;

/// Build a single-sheet Excel workbook with the given header and data rows.
///
/// * `header` — column headings, written as the first row.
/// * `rows`   — one nested vector per data row; every cell is written as a
///   string.
///
/// The workbook contains a single worksheet named `"Schedule"`. The resulting
/// `.xlsx` file is returned as raw bytes (a ZIP archive whose local-file magic
/// is the ASCII `PK`).
pub fn schedule_to_xlsx_bytes(header: &[&str], rows: &[Vec<String>]) -> Result<Vec<u8>> {
    use rust_xlsxwriter::Workbook;

    let mut workbook = Workbook::new();
    let worksheet = workbook
        .add_worksheet()
        .set_name("Schedule")
        .map_err(|e| format!("failed to create Schedule worksheet: {e}"))?;

    // Header row.
    for (col, heading) in header.iter().enumerate() {
        worksheet
            .write_string(0, col as u16, *heading)
            .map_err(|e| format!("failed to write header cell [0, {col}]: {e}"))?;
    }

    // Data rows.
    for (row, cells) in rows.iter().enumerate() {
        for (col, value) in cells.iter().enumerate() {
            worksheet
                .write_string(row as u32 + 1, col as u16, value)
                .map_err(|e| {
                    format!(
                        "failed to write data cell [{}, {}]: {e}",
                        row as u32 + 1,
                        col as u32
                    )
                })?;
        }
    }

    Ok(workbook
        .save_to_buffer()
        .map_err(|e| format!("failed to serialize xlsx workbook: {e}"))?)
}

/// Build an Excel workbook for an [`crate::electrical::ElectricalSchedule`].
///
/// The worksheet (named `"Schedule"`) has a header row and one data row per
/// record with columns:
/// `component_id,device_type,power_w,power_kw,voltage_v,current_a,power_factor,frequency_hz`.
/// The `current_a` column shows the stored current when present and otherwise
/// falls back to the value computable from power / (voltage · power factor);
/// missing optional values are written as empty cells.
pub fn electrical_schedule_to_xlsx(
    schedule: &crate::electrical::ElectricalSchedule,
) -> Result<Vec<u8>> {
    use rust_xlsxwriter::Workbook;

    let header = [
        "component_id",
        "device_type",
        "power_w",
        "power_kw",
        "voltage_v",
        "current_a",
        "power_factor",
        "frequency_hz",
    ];

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(schedule.len());
    for e in schedule.iter() {
        // Mirrors `electrical_as_csv`: use the stored current when present,
        // otherwise fall back to the value computable from the nameplate
        // voltage and power factor (I = P / (U · cos φ)).
        let current = match e.current_a {
            Some(amps) => amps.to_string(),
            None => match (e.voltage_v, e.power_factor) {
                (Some(v), Some(pf)) if v > 0.0 && pf > 0.0 => (e.power_w / (v * pf)).to_string(),
                _ => String::new(),
            },
        };
        rows.push(vec![
            e.component_id.clone(),
            e.device_type.clone(),
            e.power_w.to_string(),
            e.power_kw().to_string(),
            e.voltage_v.map(|v| v.to_string()).unwrap_or_default(),
            current,
            e.power_factor.map(|v| v.to_string()).unwrap_or_default(),
            e.frequency_hz.map(|v| v.to_string()).unwrap_or_default(),
        ]);
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook
        .add_worksheet()
        .set_name("Schedule")
        .map_err(|e| format!("failed to create Schedule worksheet: {e}"))?;

    for (col, heading) in header.iter().enumerate() {
        worksheet
            .write_string(0, col as u16, *heading)
            .map_err(|e| format!("failed to write header cell [0, {col}]: {e}"))?;
    }

    for (row, cells) in rows.iter().enumerate() {
        for (col, value) in cells.iter().enumerate() {
            worksheet
                .write_string(row as u32 + 1, col as u16, value)
                .map_err(|e| {
                    format!(
                        "failed to write data cell [{}, {}]: {e}",
                        row as u32 + 1,
                        col as u32
                    )
                })?;
        }
    }

    Ok(workbook
        .save_to_buffer()
        .map_err(|e| format!("failed to serialize xlsx workbook: {e}"))?)
}

#[cfg(all(test, feature = "export"))]
mod tests {
    use super::*;
    use crate::electrical::{ElectricalData, ElectricalSchedule};

    #[test]
    fn workbook_bytes_start_with_zip_magic() {
        let bytes = schedule_to_xlsx_bytes(&["a", "b"], &[vec!["1".into(), "2".into()]]).unwrap();
        // .xlsx files are ZIP archives and must begin with the "PK" magic.
        assert!(bytes.starts_with(b"PK"));
    }

    #[test]
    fn electrical_schedule_produces_non_empty_workbook() {
        let mut schedule = ElectricalSchedule::new();
        let mut fan = ElectricalData::new("F-01", "supply fan", 5_500.0).unwrap();
        fan.voltage_v = Some(400.0);
        fan.power_factor = Some(0.85);
        fan.frequency_hz = Some(50.0);
        schedule.add(fan);
        schedule.add(ElectricalData::new("H-01", "electric heater", 2_000.0).unwrap());

        let bytes = electrical_schedule_to_xlsx(&schedule).unwrap();
        assert!(bytes.starts_with(b"PK"));
        assert!(!bytes.is_empty());
        assert!(bytes.len() > 100);
    }
}
