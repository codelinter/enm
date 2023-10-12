pub fn microsoft_prod_patch_path(path: &str) -> Option<String> {
    if !cfg!(windows) {
        return None;
    }

    let output = std::process::Command::new("cygpath")
        .arg(path)
        .output()
        .ok()?;
    if output.status.success() {
        let output = String::from_utf8(output.stdout).ok()?;
        Some(output.trim().to_string())
    } else {
        None
    }
}
