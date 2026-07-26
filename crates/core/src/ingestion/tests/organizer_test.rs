use crate::ingestion::organizer::{Organizer, OrganizedOutput};
use crate::ingestion::mineru_wrapper::MinerUOutput;
use std::fs;
use tempfile::tempdir;
use uuid::Uuid;

fn make_auto_dir(auto_dir: &std::path::Path, md_name: &str, images: &[&str]) -> std::path::PathBuf {
    fs::create_dir_all(auto_dir).unwrap();
    let md_path = auto_dir.join(md_name);
    let mut content = String::new();
    content.push_str("# Book Title\n\nSome text.\n\n");
    for img in images {
        content.push_str(&format!("![desc]({})\n", img));
    }
    fs::write(&md_path, content).unwrap();

    let images_dir = auto_dir.join("images");
    fs::create_dir_all(&images_dir).unwrap();
    for img in images {
        fs::write(images_dir.join(img), "PNGDATA").unwrap();
    }

    md_path
}

fn make_output(auto_dir: std::path::PathBuf, markdown_path: std::path::PathBuf) -> MinerUOutput {
    MinerUOutput {
        job_id: Uuid::new_v4(),
        markdown_path,
        images_dir: Some(auto_dir.join("images")),
        auto_dir,
        method: crate::ingestion::models::ProcessingMethod::MinerU {
            gpu: true,
            model_version: "v1".to_string(),
        },
    }
}

fn make_book_root(temp: &tempfile::TempDir) -> std::path::PathBuf {
    temp.path().join("book")
}

#[test]
fn test_extract_image_paths() {
    let temp = tempdir().unwrap();
    let auto_dir = temp.path().join("auto");
    let md_path = make_auto_dir(&auto_dir, "chapter.md", &["img1.png", "img2.jpg"]);

    let organizer = Organizer::new(temp.path().to_path_buf());
    let paths = organizer.extract_image_paths(&md_path).unwrap();

    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.ends_with("img1.png")));
    assert!(paths.iter().any(|p| p.ends_with("img2.jpg")));
}

#[tokio::test]
async fn test_reorganize_creates_clean_structure() {
    let temp = tempdir().unwrap();
    let auto_dir = temp.path().join("auto");
    let md_path = make_auto_dir(&auto_dir, "chapter.md", &["img1.png"]);
    let output = make_output(auto_dir.clone(), md_path.clone());
    let book_root = make_book_root(&temp);

    let organizer = Organizer::new(temp.path().to_path_buf());
    let result = organizer.reorganize(&output, &book_root).await.unwrap();

    assert!(result.markdown_path.exists());
    assert!(result.images_dir.exists());
    assert_eq!(result.images_moved, 1);

    let moved_img = result.images_dir.join("img1.png");
    assert!(moved_img.exists(), "Image should be moved: {:?}", moved_img);
}

#[tokio::test]
async fn test_reorganize_cleans_auto_dir() {
    let temp = tempdir().unwrap();
    let auto_dir = temp.path().join("auto");
    let md_path = make_auto_dir(&auto_dir, "chapter.md", &[]);
    let output = make_output(auto_dir.clone(), md_path.clone());
    let book_root = make_book_root(&temp);

    let organizer = Organizer::new(temp.path().to_path_buf());
    let _ = organizer.reorganize(&output, &book_root).await.unwrap();

    assert!(!auto_dir.exists(), "auto/ dir should be cleaned up");
}

#[test]
fn test_organizer_creates_log() {
    let temp = tempdir().unwrap();
    let auto_dir = temp.path().join("auto");
    let md_path = make_auto_dir(&auto_dir, "chapter.md", &[]);
    let output = make_output(auto_dir, md_path);
    let book_root = make_book_root(&temp);

    let organizer = Organizer::new(temp.path().to_path_buf());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(organizer.reorganize(&output, &book_root)).unwrap();

    let log_dir = temp.path().join("_reorg_logs");
    assert!(log_dir.exists());
    let logs: Vec<_> = fs::read_dir(&log_dir).unwrap().collect();
    assert!(!logs.is_empty());
}
