//! Smoke test: the generated PDF parses with printpdf's own parser.
#![cfg(feature = "export")]

use venti::{electrical_schedule_to_pdf, schedule_to_pdf_bytes};

#[test]
fn pdf_parses_roundtrip() {
    let bytes = schedule_to_pdf_bytes(
        &["component_id", "device_type", "power_w"],
        &[vec!["F-01".into(), "supply fan".into(), "5500".into()]],
    )
    .unwrap();
    let doc = printpdf::PdfDocument::parse(
        &bytes,
        &printpdf::PdfParseOptions::default(),
        &mut Vec::new(),
    );
    let doc = doc.expect("generated PDF must parse");
    assert_eq!(doc.pages.len(), 1, "single-page PDF");
    let text = doc.pages[0].extract_text(&doc.resources).join(" ");
    assert!(text.contains("Schedule"), "title present");
    assert!(text.contains("component_id"), "header present");
    assert!(text.contains("F-01"), "data present");
}

#[test]
fn electrical_pdf_parses_roundtrip() {
    use venti::{ElectricalData, ElectricalSchedule};
    let mut s = ElectricalSchedule::new();
    s.add(ElectricalData::new("F-01", "supply fan", 5500.0).unwrap());
    let bytes = electrical_schedule_to_pdf(&s).unwrap();
    let doc = printpdf::PdfDocument::parse(
        &bytes,
        &printpdf::PdfParseOptions::default(),
        &mut Vec::new(),
    )
    .expect("generated PDF must parse");
    assert_eq!(doc.pages.len(), 1);
    assert!(doc.pages[0]
        .extract_text(&doc.resources)
        .join(" ")
        .contains("F-01"));
}
