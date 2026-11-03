use std::env;
use std::path::PathBuf;

pub const ROOT_VAR: &str = "MINIRUNC_ROOT";
pub const DEFAULT_ROOT: &str = "/run/minirunc";

/// Returns the given root, or the default when none is set.
pub fn root_dir(value: Option<String>) -> PathBuf {
    PathBuf::from(value.unwrap_or_else(|| DEFAULT_ROOT.to_string()))
}

/// Returns `<root>/<path>`, using MINIRUNC_ROOT or the default root.
pub fn root_path(path: &str) -> PathBuf {
    root_dir(env::var(ROOT_VAR).ok()).join(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_default_when_not_set() {
        assert_eq!(root_dir(None), PathBuf::from(DEFAULT_ROOT));
    }

    #[test]
    fn uses_value_when_set() {
        assert_eq!(root_dir(Some("/tmp/x".into())), PathBuf::from("/tmp/x"));
    }
}