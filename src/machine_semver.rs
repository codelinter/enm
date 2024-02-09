use std::path::PathBuf;

pub fn path() -> PathBuf {
    let path_as_string = if cfg!(windows) {
        "A:/_enm_/X/Y/Z/A/installation"
    } else {
        "/_enm_/X/Y/Z/A/installation"
    };

    PathBuf::from(path_as_string)
}

pub fn display_name() -> &'static str {
    "system"
}
