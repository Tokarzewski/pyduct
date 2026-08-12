//! Report export engine: Excel (`.xlsx`) and PDF.
//!
//! This module turns in-memory report tables and the [`crate::electrical`]
//! schedule into binary Excel workbooks and single-page PDF documents,
//! completing issue #45 (ModelTok/pyduct).
//!
//! - `.xlsx` output uses `rust_xlsxwriter`.
//! - `.pdf` output uses `printpdf` with its built-in Helvetica fonts, so no
//!   font files need to ship with the crate.
//!
//! The module is gated behind the `export` cargo feature so that the crate
//! core (`cargo build --no-default-features --lib`) stays dependency-free:
//! the dependencies it introduces, `rust_xlsxwriter` and `printpdf`, are
//! optional and only pulled in when the feature is enabled.

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

// ---------------------------------------------------------------------------
// PDF renderer (printpdf, built-in Helvetica fonts)
// ---------------------------------------------------------------------------

/// Build a single-page PDF with the given title, bold header row and data
/// rows, laid out on an A4 portrait sheet.
///
/// Text is rendered with printpdf's built-in Helvetica font family
/// (Helvetica for the body, Helvetica-Bold for the title and header), so no
/// font files are needed. Column positions are spread evenly across the page
/// width; the header row sits on a light-grey band. The resulting PDF is
/// returned as raw bytes (magic `%PDF`).
fn render_table_pdf(title: &str, header: &[&str], rows: &[Vec<String>]) -> Result<Vec<u8>> {
    use printpdf::{BuiltinFont, Color, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, Point, Pt};

    // A4 portrait: 210 x 297 mm. Margin 15 mm, usable width 180 mm.
    let margin_mm = 15.0_f64;
    let width_mm = 210.0_f64;
    let height_mm = 297.0_f64;
    let usable_mm = width_mm - 2.0 * margin_mm;
    let col_xs: Vec<f64> = if header.is_empty() {
        Vec::new()
    } else {
        let step = usable_mm / header.len() as f64;
        (0..header.len())
            .map(|i| margin_mm + i as f64 * step)
            .collect()
    };

    // Draw one text run: move the cursor to (x, y) and emit the cell.
    fn cell_ops(x: f64, y_mm: f64, text: &str) -> Vec<Op> {
        use printpdf::TextItem;
        vec![
            Op::SetTextCursor {
                pos: Point::new(Mm(x as f32), Mm(y_mm as f32)),
            },
            Op::ShowText {
                items: vec![TextItem::Text(text.to_string())],
            },
        ]
    }

    let mut ops: Vec<Op> = Vec::new();
    ops.push(Op::StartTextSection);

    // Title.
    let title_y = height_mm - 18.0;
    ops.push(Op::SetFont {
        font: PdfFontHandle::Builtin(BuiltinFont::HelveticaBold),
        size: Pt(16.0),
    });
    ops.push(Op::SetFillColor {
        col: Color::Rgb(printpdf::Rgb {
            r: 0.12,
            g: 0.25,
            b: 0.5,
            icc_profile: None,
        }),
    });
    ops.extend(cell_ops(margin_mm, title_y, title));

    // Header row on a light-grey band.
    let header_y = height_mm - 32.0;
    ops.push(Op::SetFillColor {
        col: Color::Rgb(printpdf::Rgb {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            icc_profile: None,
        }),
    });
    ops.push(Op::DrawRectangle {
        rectangle: printpdf::Rect::from_xywh(
            Mm(margin_mm as f32).into(),
            Mm((header_y - 4.0) as f32).into(),
            Mm(usable_mm as f32).into(),
            Mm(7.0).into(),
        ),
    });
    ops.push(Op::SetFillColor {
        col: Color::Rgb(printpdf::Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::Builtin(BuiltinFont::HelveticaBold),
        size: Pt(9.5),
    });
    for (i, heading) in header.iter().enumerate() {
        let x = *col_xs.get(i).unwrap_or(&margin_mm);
        ops.extend(cell_ops(x, header_y, heading));
    }

    // Data rows.
    ops.push(Op::SetFont {
        font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
        size: Pt(9.0),
    });
    let row_gap_mm = 5.5_f64;
    let mut y = header_y - 9.0;
    for cells in rows {
        for (i, value) in cells.iter().enumerate() {
            let x = *col_xs.get(i).unwrap_or(&margin_mm);
            ops.extend(cell_ops(x, y, value));
        }
        y -= row_gap_mm;
    }
    ops.push(Op::EndTextSection);

    let mut doc = PdfDocument::new(title);
    doc.pages = vec![PdfPage::new(Mm(width_mm as f32), Mm(height_mm as f32), ops)];
    Ok(doc.save(&printpdf::PdfSaveOptions::default(), &mut Vec::new()))
}

/// Build a single-page PDF report with the given header and data rows.
///
/// * `header` — column headings, written as a bold first row.
/// * `rows`   — one nested vector per data row; every cell is written as a
///   string.
///
/// The document is a single A4 portrait page titled `"Schedule"` and uses
/// printpdf's built-in Helvetica fonts. The resulting PDF is returned as raw
/// bytes (magic `%PDF`).
pub fn schedule_to_pdf_bytes(header: &[&str], rows: &[Vec<String>]) -> Result<Vec<u8>> {
    render_table_pdf("Schedule", header, rows)
}

/// Build a single-page PDF report for an [`crate::electrical::ElectricalSchedule`].
///
/// The document has a bold header row and one data row per record with
/// columns:
/// `component_id,device_type,power_w,power_kw,voltage_v,current_a,power_factor,frequency_hz`.
/// The `current_a` column shows the stored current when present and otherwise
/// falls back to the value computable from power / (voltage · power factor);
/// missing optional values are written as empty cells. The rendering mirrors
/// [`electrical_schedule_to_xlsx`].
pub fn electrical_schedule_to_pdf(
    schedule: &crate::electrical::ElectricalSchedule,
) -> Result<Vec<u8>> {
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
        // Same fallback as `electrical_schedule_to_xlsx` / `electrical_as_csv`:
        // use the stored current when present, otherwise derive I = P / (U · cos φ).
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

    render_table_pdf("Electrical Schedule", &header, &rows)
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

    #[test]
    fn pdf_bytes_start_with_pdf_magic() {
        let bytes = schedule_to_pdf_bytes(&["a", "b"], &[vec!["1".into(), "2".into()]]).unwrap();
        // PDF files must begin with the ASCII "%PDF" magic.
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
        assert!(bytes.len() > 100);
    }

    #[test]
    fn electrical_schedule_produces_pdf() {
        let mut schedule = ElectricalSchedule::new();
        let mut fan = ElectricalData::new("F-01", "supply fan", 5_500.0).unwrap();
        fan.voltage_v = Some(400.0);
        fan.power_factor = Some(0.85);
        fan.frequency_hz = Some(50.0);
        schedule.add(fan);
        schedule.add(ElectricalData::new("H-01", "electric heater", 2_000.0).unwrap());

        let bytes = electrical_schedule_to_pdf(&schedule).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
        assert!(bytes.len() > 100);
    }
}
