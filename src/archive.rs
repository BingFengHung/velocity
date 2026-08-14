use crate::fs::format_size;
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

#[derive(Clone, Debug)]
pub struct ArchivePreviewInfo {
    pub total_files: usize,
    pub uncompressed_size: u64,
    pub items: Vec<String>,
}

pub fn read_zip_preview(path: &Path, max_items: usize) -> Result<ArchivePreviewInfo, String> {
    let file = File::open(path).map_err(|e| format!("無法開啟壓縮檔: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("無法讀取壓縮檔結構: {}", e))?;

    let total_files = archive.len();
    let mut uncompressed_size: u64 = 0;
    let mut items = Vec::new();

    for i in 0..total_files {
        if let Ok(file) = archive.by_index(i) {
            uncompressed_size += file.size();
            if items.len() < max_items {
                let is_dir = file.is_dir();
                let icon = if is_dir { "[DIR]" } else { "     " };
                let size_str = if is_dir {
                    String::new()
                } else {
                    format!(" ({})", format_size(file.size()))
                };
                items.push(format!("{} {}{}", icon, file.name(), size_str));
            }
        }
    }

    Ok(ArchivePreviewInfo {
        total_files,
        uncompressed_size,
        items,
    })
}
