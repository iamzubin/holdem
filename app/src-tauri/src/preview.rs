//! Space-bar preview of a shelf item: the image itself, the start of a text
//! file, a folder's contents, or Windows' own thumbnail for everything else.

use crate::FileList;
use base64::Engine;
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::State;

/// Images larger than this are shown as Windows' thumbnail instead.
const MAX_IMAGE_BYTES: u64 = 30 * 1024 * 1024;
/// How much of a text file is shown.
const MAX_TEXT_BYTES: u64 = 200 * 1024;
const MAX_FOLDER_ENTRIES: usize = 200;

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Preview {
    Image { mime: &'static str, data: String },
    Text { text: String, truncated: bool },
    Folder { entries: Vec<FolderEntry>, total: usize },
    Thumbnail { data: String },
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FolderEntry {
    name: String,
    folder: bool,
}

fn image_mime(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "png" => "image/png",
        "jpg" | "jpeg" | "jfif" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        // Shown in an <img>, where SVG scripts never run.
        "svg" => "image/svg+xml",
        _ => return None,
    })
}

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "log", "csv", "tsv", "json", "jsonc", "xml", "yaml", "yml", "toml",
    "ini", "cfg", "conf", "env", "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "rb", "go",
    "java", "kt", "c", "h", "cpp", "hpp", "cs", "swift", "php", "html", "htm", "css", "scss",
    "sql", "sh", "bash", "ps1", "bat", "cmd", "lua", "r", "gitignore", "editorconfig", "url",
];

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

fn read_text(path: &Path) -> Result<Preview, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut bytes = Vec::new();
    file.take(MAX_TEXT_BYTES).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    Ok(Preview::Text {
        text: String::from_utf8_lossy(&bytes).into_owned(),
        truncated: size > MAX_TEXT_BYTES,
    })
}

fn read_folder(path: &Path) -> Result<Preview, String> {
    let mut entries: Vec<FolderEntry> = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| FolderEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            folder: entry.file_type().is_ok_and(|t| t.is_dir()),
        })
        .collect();
    let total = entries.len();
    // Folders first, then by name, as Explorer lists them.
    entries.sort_by(|a, b| {
        b.folder.cmp(&a.folder).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries.truncate(MAX_FOLDER_ENTRIES);
    Ok(Preview::Folder { entries, total })
}

fn build_preview(path: &Path, is_note: bool) -> Result<Preview, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.is_dir() {
        return read_folder(path);
    }
    let ext = extension(path);
    if let Some(mime) = image_mime(&ext) {
        if meta.len() <= MAX_IMAGE_BYTES {
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let data = base64::engine::general_purpose::STANDARD.encode(bytes);
            return Ok(Preview::Image { mime, data });
        }
    }
    if is_note || TEXT_EXTENSIONS.contains(&ext.as_str()) {
        return read_text(path);
    }
    crate::thumbnail::get_thumbnail_base64(path).map(|data| Preview::Thumbnail { data })
}

/// Preview of one shelf item, by id (the page never names a path).
#[tauri::command]
pub async fn get_preview(file_list: State<'_, FileList>, id: u64) -> Result<Preview, String> {
    let (path, is_note): (PathBuf, bool) = {
        let list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        let file = list.iter().find(|f| f.id == id).ok_or("Not on the shelf")?;
        if !file.available {
            return Err("The item's drive or network location is offline".to_string());
        }
        (file.path.clone(), file.kind == "note")
    };
    tauri::async_runtime::spawn_blocking(move || build_preview(&path, is_note))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("holdem_preview_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("dir");
        dir
    }

    #[test]
    fn text_is_read_and_capped() {
        let dir = temp_dir();
        let small = dir.join("a.md");
        std::fs::write(&small, "# hi").expect("write");
        assert_eq!(
            build_preview(&small, false).expect("preview"),
            Preview::Text { text: "# hi".into(), truncated: false }
        );
        let big = dir.join("b.log");
        std::fs::write(&big, vec![b'x'; MAX_TEXT_BYTES as usize + 10]).expect("write");
        match build_preview(&big, false).expect("preview") {
            Preview::Text { text, truncated } => {
                assert!(truncated);
                assert_eq!(text.len(), MAX_TEXT_BYTES as usize);
            }
            other => panic!("unexpected {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn folders_list_subfolders_first() {
        let dir = temp_dir();
        std::fs::write(dir.join("a.txt"), "x").expect("write");
        std::fs::create_dir(dir.join("z sub")).expect("sub");
        match build_preview(&dir, false).expect("preview") {
            Preview::Folder { entries, total } => {
                assert_eq!(total, 2);
                assert_eq!(entries[0], FolderEntry { name: "z sub".into(), folder: true });
            }
            other => panic!("unexpected {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn images_are_sent_as_is() {
        let dir = temp_dir();
        let png = dir.join("p.PNG");
        std::fs::write(&png, b"\x89PNG\r\n").expect("write");
        assert!(matches!(
            build_preview(&png, false).expect("preview"),
            Preview::Image { mime: "image/png", .. }
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
