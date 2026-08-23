use pdf_inspector_skillkit::{classify, process, validate_path, PdfInfo, SkillkitError};
use std::path::PathBuf;

/// A redistributable U.S. Code fixture tracked in this repository.
///
/// Tests must never discover arbitrary documents from a contributor's home
/// directory: doing so is nondeterministic and can process private files.
fn public_test_pdf() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-corpus/source/sample-1.pdf")
}

#[test]
fn test_classify_text_pdf() {
    let pdf_path = public_test_pdf();
    assert!(pdf_path.is_file(), "public test fixture is missing");
    let info = classify(&pdf_path).expect("classify failed");
    assert_eq!(info.pdf_type, "TextBased");
    assert_eq!(info.page_count, 4);
}

#[test]
fn test_classify_nonexistent() {
    let result = classify("/nonexistent.pdf");
    assert!(matches!(result, Err(SkillkitError::FileNotFound(_))));
}

#[test]
fn test_process_produces_markdown() {
    let pdf_path = public_test_pdf();
    let info = process(&pdf_path).expect("process failed");
    assert!(info.markdown.is_some(), "markdown should be Some");
    let markdown = info.markdown.as_deref().unwrap();
    assert!(
        markdown.contains("§1398"),
        "expected public fixture content"
    );
}

#[test]
fn test_validate_path_accepts_public_fixture() {
    let pdf_path = public_test_pdf();
    let result = validate_path(&pdf_path);
    assert!(result.is_ok(), "valid PDF path should pass validation");
}

#[test]
fn test_validate_path_canonicalizes() {
    let result = validate_path(public_test_pdf()).expect("validate_path failed");
    assert!(result.is_absolute(), "should return absolute path");
}

#[test]
fn test_pdf_info_serialization() {
    let info = PdfInfo {
        pdf_type: "TextBased".to_string(),
        confidence: 0.95,
        page_count: 10,
        pages_needing_ocr: vec![],
        has_encoding_issues: false,
        title: Some("Test Document".to_string()),
        markdown: Some("# Test\n\nHello world".to_string()),
        processing_time_ms: 123,
    };
    let json = serde_json::to_string(&info).expect("serialize failed");
    assert!(json.contains("\"pdf_type\""));
    assert!(json.contains("\"confidence\""));
    assert!(json.contains("\"page_count\""));
    assert!(json.contains("\"pages_needing_ocr\""));
    assert!(json.contains("\"has_encoding_issues\""));
    assert!(json.contains("\"title\""));
    assert!(json.contains("\"markdown\""));
    assert!(json.contains("\"processing_time_ms\""));
}
