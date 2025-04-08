// src/crawler.rs

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use tracing::info;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::app::AppMessage;

/// Filter function that returns `false` if the path should be skipped.
fn filter_entry(entry: &DirEntry, ignore_list: &[String]) -> bool {
    let path_str = entry.path().display().to_string();

    // If any ignore pattern is found in the path, skip:
    !ignore_list.iter().any(|pattern| path_str.contains(pattern))
}

/// Collects *all* descendant directories from a root, sending them line-by-line,
/// but skipping any path containing an ignore pattern.
pub fn gather_descendant_dirs_streaming(
    root_path: PathBuf,
    tx: &Sender<AppMessage>,
    ignore_list: &[String],
) {
    info!(
        "Starting to gather descendant directories from: {:?}",
        root_path
    );
    // Use .filter_entry() to prune directories we want to ignore
    let walker = WalkDir::new(&root_path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| filter_entry(e, ignore_list));

    for entry_result in walker {
        match entry_result {
            Ok(entry) if entry.file_type().is_dir() => {
                // If the receiver side is closed, break
                if tx
                    .send(AppMessage::Subdir(entry.path().to_path_buf()))
                    .is_err()
                {
                    break;
                }
            }
            // We ignore files, but you could also track them if needed
            _ => {}
        }
    }
    info!("Finished gathering directories from: {:?}", root_path);
}
