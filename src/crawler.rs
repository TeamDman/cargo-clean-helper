use walkdir::WalkDir;

/// Collects *all* descendant directories starting from a given path.
/// e.g. `gather_descendant_dirs("C:\\MyFolder")` -> Vector of subdirectory paths.
pub fn gather_descendant_dirs(root_path: &str) -> Vec<String> {
    let mut dirs = Vec::new();

    // The WalkDir builder can be customized with filters, e.g. ignoring hidden dirs, etc.
    let walker = WalkDir::new(root_path).follow_links(true).into_iter();

    for entry in walker.flatten() {
        if entry.file_type().is_dir() {
            // Convert to a user-friendly string
            let path_str = entry.path().display().to_string();
            dirs.push(path_str);
        }
    }

    dirs
}
