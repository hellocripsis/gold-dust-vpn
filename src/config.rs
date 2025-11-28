use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Simple on/off flags for Oxen and Tor backends.
///
/// In a real system this would hold addresses, keys, and more,
/// but for v0.1 we only need feature toggles.
#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    pub oxen_enabled: bool,
    pub tor_enabled: bool,
}

/// Top-level Gold Dust config structure.
///
/// Loaded from `gold-dust-vpn.toml` via `toml` + `serde`.
#[derive(Debug, Clone, Deserialize)]
pub struct GoldDustConfig {
    pub backends: BackendConfig,
}

impl GoldDustConfig {
    /// Load config from a TOML file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path)?;
        let cfg: GoldDustConfig = toml::from_str(&text)?;
        Ok(cfg)
    }
}
