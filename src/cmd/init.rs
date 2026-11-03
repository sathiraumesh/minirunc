use anyhow::Result;

/// Entry point for `minirunc init`. Only Linux has namespaces, so on other
/// systems this just reports an error.
pub fn run(id: &str, bundle: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    return crate::os::init(id, std::path::Path::new(bundle));

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (id, bundle);
        anyhow::bail!("init is only supported on Linux")
    }
}
