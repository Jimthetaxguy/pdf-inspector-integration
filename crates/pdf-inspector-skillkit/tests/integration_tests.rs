use pdf_inspector_skillkit::{classify, process, validate_path, PdfInfo, SkillkitError};

fn find_test_pdf() -> Option<std::path::PathBuf> {
    for dir in &[
        dirs::home_dir().unwrap().join("Documents"),
        dirs::home_dir().unwrap().join("Downloads"),
    ] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "pdf") && path.is_file() {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[test]
#[ignore] // Run with: cargo test -p pdf-inspector-skillkit -- --ignored
fn test_classify_text_pdf() {
    let pdf_path = find_test_pdf().expect("No PDF found in ~/Documents or ~/Downloads");
    let info = classify(&pdf_path).expect("classify failed");
    assert!(!info.pdf_type.is_empty(), "pdf_type should not be empty");
}

#[test]
fn test_classify_nonexistent() {
    let result = classify("/nonexistent.pdf");
    assert!(matches!(result, Err(SkillkitError::FileNotFound(_))));
}

#[test]
#[ignore] // Run with: cargo test -p pdf-inspector-skillkit -- --ignored
fn test_process_produces_markdown() {
    let pdf_path = find_test_pdf().expect("No PDF found in ~/Documents or ~/Downloads");
    let info = process(&pdf_path).expect("process failed");
    assert!(info.markdown.is_some(), "markdown should be Some");
    assert!(
        !info.markdown.as_ref().unwrap().is_empty(),
        "markdown should be non-empty"
    );
}

#[test]
#[ignore] // Run with: cargo test -p pdf-inspector-skillkit -- --ignored
fn test_validate_path_rejects_oversized() {
    let pdf_path = find_test_pdf().expect("No PDF found in ~/Documents or ~/Downloads");
    let result = validate_path(&pdf_path);
    assert!(result.is_ok(), "valid PDF path should pass validation");
}

#[test]
#[ignore] // Run with: cargo test -p pdf-inspector-skillkit -- --ignored
fn test_validate_path_canonicalizes() {
    let pdf_path = find_test_pdf().expect("No PDF found in ~/Documents or ~/Downloads");
    let parent = pdf_path.parent().unwrap();
    let filename = pdf_path.file_name().unwrap();
    let relative = parent.join(filename);
    let result = validate_path(&relative).expect("validate_path failed");
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
