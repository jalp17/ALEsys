use crate::ingestion::pymupdf_fallback::PyMuPDFFallback;
use crate::ingestion::models::{Chapter, ImageRef, IngestionMode};
use std::path::PathBuf;
use uuid::Uuid;

fn make_fallback() -> PyMuPDFFallback {
    PyMuPDFFallback::new()
}

#[test]
fn test_fallback_new() {
    let fb = make_fallback();
}

#[test]
fn test_build_chapters_with_headers() {
    let fb = make_fallback();
    let pages = vec![
        crate::ingestion::pymupdf_fallback::PageText {
            page_num: 1,
            text: "# Introduction\n\nThis is intro.\n\n# Background\n\nThis is background.\n".to_string(),
            tables: vec![],
        },
    ];
    let images = vec![];
    let chapters = fb.build_chapters(&pages, &images);
    assert_eq!(chapters.len(), 2);
    assert_eq!(chapters[0].title, "Introduction");
    assert_eq!(chapters[1].title, "Background");
    assert_eq!(chapters[0].start_page, 1);
}

#[test]
fn test_build_chapters_without_headers() {
    let fb = make_fallback();
    let pages = vec![crate::ingestion::pymupdf_fallback::PageText {
        page_num: 1,
        text: "Just some text without headers.\n".to_string(),
        tables: vec![],
    }];
    let images = vec![];
    let chapters = fb.build_chapters(&pages, &images);
    assert_eq!(chapters.len(), 1);
    assert_eq!(chapters[0].title, "Document");
}

#[test]
fn test_generate_markdown() {
    let fb = make_fallback();
    let chapters = vec![Chapter {
        id: Uuid::new_v4(),
        title: "Test".to_string(),
        level: 1,
        start_page: 1,
        end_page: 2,
        markdown_path: PathBuf::new(),
        image_refs: vec![],
    }];
    let images = vec![];
    let md = fb.generate_markdown(&chapters, &images);
    assert!(md.contains("# Test"));
    assert!(md.contains("Pages 1-2"));
}

#[test]
fn test_parse_pdfplumber_output() {
    let fb = make_fallback();
    let output = "---PAGE 0---\nHello world\n---TABLES 1---\na|b\n1|2\n---PAGE 1---\nSecond page\n";
    let pages = fb.parse_pdfplumber_output(output);
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].text, "Hello world\n");
    assert_eq!(pages[0].tables.len(), 2);
    assert_eq!(pages[1].text, "Second page\n");
}
