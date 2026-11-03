use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const CONFIG_FILE: &str = "config.json";

// The OCI config.json from a bundle (only the fields minirunc uses).
// `#[serde(default)]` mimics Go: a missing field becomes its zero value.

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Spec {
    pub hostname: String,
    pub process: Process,
    pub root: Root,
    pub linux: Linux,
}

impl Spec {
    /// Reads and parses `<bundle>/config.json`.
    pub fn load(bundle: &Path) -> Result<Spec> {
        let path = bundle.join(CONFIG_FILE);

        let text = fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;

        let spec: Spec = serde_json::from_str(&text)
            .with_context(|| format!("parse {}", path.display()))?;

        spec.validate()
            .with_context(|| format!("invalid {}", path.display()))?;

        Ok(spec)
    }

    /// Checks the fields a container can't run without.
    fn validate(&self) -> Result<()> {
        if self.process.args.is_empty() {
            bail!("process.args is empty");
        }
        if self.root.path.is_empty() {
            bail!("root.path is empty");
        }
        Ok(())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Process {
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub cwd: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Root {
    pub path: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Linux {
    pub namespaces: Vec<Namespace>,
    pub resources: Resources,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Namespace {
    #[serde(rename = "type")] // `type` is a Rust keyword
    pub kind: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Resources {
    pub memory: Memory,
    pub cpu: Cpu,
    pub pids: Pids,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Memory {
    /// Bytes.
    pub limit: i64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Cpu {
    /// Microseconds per period.
    pub quota: i64,
    /// Microseconds.
    pub period: u64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Pids {
    pub limit: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config() {
        let json = r#"{
            "hostname": "box",
            "process": { "args": ["/bin/sh"], "env": ["PATH=/bin"], "cwd": "/" },
            "root": { "path": "rootfs" },
            "linux": {
                "namespaces": [{ "type": "pid" }, { "type": "mount" }],
                "resources": { "memory": { "limit": 1048576 }, "cpu": { "quota": 50000, "period": 100000 } }
            }
        }"#;

        let spec: Spec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.hostname, "box");
        assert_eq!(spec.process.args, ["/bin/sh"]);
        assert_eq!(spec.root.path, "rootfs");
        assert_eq!(spec.linux.namespaces[1].kind, "mount");
        assert_eq!(spec.linux.resources.memory.limit, 1048576);
        assert_eq!(spec.linux.resources.pids.limit, 0); // missing → default
    }

    #[test]
    fn load_reads_bundle() {
        let bundle = std::env::temp_dir().join(format!("minirunc-spec-{}", std::process::id()));
        fs::create_dir_all(&bundle).unwrap();

        fs::write(bundle.join(CONFIG_FILE), r#"{"process":{"args":["/bin/sh"]},"root":{"path":"rootfs"}}"#).unwrap();
        let spec = Spec::load(&bundle).unwrap();
        assert_eq!(spec.root.path, "rootfs");

        fs::write(bundle.join(CONFIG_FILE), r#"{"root":{"path":"rootfs"}}"#).unwrap();
        assert!(Spec::load(&bundle).is_err()); // no process.args

        fs::remove_dir_all(&bundle).unwrap();
        assert!(Spec::load(&bundle).is_err()); // no config.json
    }
}
