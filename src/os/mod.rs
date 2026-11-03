// OS-specific code. Namespaces and cgroups only exist on Linux,
// so this module is empty when building on other systems.

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{init, namespace_flags, spawn};
