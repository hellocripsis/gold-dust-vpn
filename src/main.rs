use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Gold Dust VPN: Oxen-first, Tor-fallback routing brain.
///
/// v0.1: health checks + "which backend would I use?" decisions.
/// This is a control plane, not a full VPN tunnel yet.

#[derive(Debug, Deserialize)]
struct BackendsConfig {
    oxen_enabled: bool,
    tor_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct GoldDustConfig {
    backends: BackendsConfig,
}

#[derive(Debug)]
enum BackendChoice {
    Oxen,
    Tor,
    None(&'static str),
}

#[derive(Parser, Debug)]
#[command(name = "gold-dust-vpn", version)]
struct Cli {
    /// Path to config file (defaults to ./gold-dust-vpn.toml)
    #[arg(long, short)]
    config: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show backend health and current preferred route
    Status,
    /// Decide how we would route a given host:port
    Route {
        /// Host:port pair, e.g. example.com:443
        target: String,
    },
}

fn load_config(path: &Path) -> anyhow::Result<GoldDustConfig> {
    let raw = fs::read_to_string(path)?;
    let cfg: GoldDustConfig = toml::from_str(&raw)?;
    Ok(cfg)
}

fn check_backends(cfg: &GoldDustConfig) -> String {
    let mut lines = Vec::new();

    if cfg.backends.oxen_enabled {
        lines.push("Oxen: enabled (stubbed healthy)".to_string());
    } else {
        lines.push("Oxen: disabled".to_string());
    }

    if cfg.backends.tor_enabled {
        lines.push("Tor: enabled (stubbed healthy)".to_string());
    } else {
        lines.push("Tor: disabled".to_string());
    }

    lines.join("\n")
}

fn choose_backend(cfg: &GoldDustConfig, _target: &str) -> BackendChoice {
    if cfg.backends.oxen_enabled {
        BackendChoice::Oxen
    } else if cfg.backends.tor_enabled {
        BackendChoice::Tor
    } else {
        BackendChoice::None("no backends enabled in config")
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let cfg_path_str = cli
        .config
        .unwrap_or_else(|| "gold-dust-vpn.toml".to_string());
    let cfg_path = Path::new(&cfg_path_str);

    let cfg = load_config(cfg_path)?;

    match cli.command {
        Command::Status => {
            let status = check_backends(&cfg);
            println!("{status}");
        }
        Command::Route { target } => {
            let choice = choose_backend(&cfg, &target);
            match choice {
                BackendChoice::Oxen => {
                    println!("Gold Dust VPN would route {target} via OXEN (primary).");
                }
                BackendChoice::Tor => {
                    println!("Gold Dust VPN would route {target} via TOR (fallback).");
                }
                BackendChoice::None(reason) => {
                    println!("No backend available for {target}: {reason}");
                }
            }
        }
    }

    Ok(())
}
