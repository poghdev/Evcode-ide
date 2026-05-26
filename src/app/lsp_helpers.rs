pub fn language_id(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default() {
        "rs" => "rust",
        "py" => "python",
        "js" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "go" => "go",
        "c" | "h" => "c",
        "cc" | "cpp" | "hpp" => "cpp",
        _ => "plaintext",
    }
}

pub fn file_uri(path: &str) -> Option<String> {
    let path = std::path::Path::new(path);
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut uri = String::from("file://");
    uri.push_str(&abs.to_string_lossy());
    Some(uri)
}