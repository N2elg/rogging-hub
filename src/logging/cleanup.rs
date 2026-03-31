/// Remove old rotated log files if they exceed `max_files`.
pub(crate) fn cleanup_old_files(dir: &str, prefix: &str, max_files: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with(prefix))
        })
        .collect();

    if files.len() <= max_files {
        return;
    }

    // Sort by modified time, oldest first.
    files.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let to_remove = files.len() - max_files;
    for entry in files.into_iter().take(to_remove) {
        let path = entry.path();
        if std::fs::remove_file(&path).is_ok() {
            tracing::info!(file = %path.display(), "Removed old log file");
        }
    }
}
