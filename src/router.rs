use crate::config::GoldDustConfig;

/// Type of backend: Oxen node or Tor exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Oxen,
    Tor,
}

/// Health snapshot for a single backend.
#[derive(Debug, Clone)]
pub struct BackendHealth {
    pub name: String,
    pub kind: BackendKind,
    pub latency_ms: f64,
    pub failure_rate: f64,
    pub enabled: bool,
}

/// Snapshot of all backends at a point in time.
#[derive(Debug, Clone)]
pub struct RouterSnapshot {
    pub backends: Vec<BackendHealth>,
}

/// Routing decision for a particular target.
#[derive(Debug, Clone)]
pub struct BackendChoice {
    pub target: String,
    pub backend: BackendHealth,
}

/// Gold Dust router: Oxen-first, Tor-fallback.
///
/// v0.1: uses static, deterministic health values and simple
/// config flags for enable/disable. In a real system this would
/// be fed by live telemetry.
#[derive(Debug)]
pub struct Router {
    cfg: GoldDustConfig,
}

impl Router {
    pub fn new(cfg: GoldDustConfig) -> Self {
        Self { cfg }
    }

    /// Build a static health snapshot, honoring config flags.
    fn sample_health(&self) -> RouterSnapshot {
        let oxen_enabled = self.cfg.backends.oxen_enabled;
        let tor_enabled = self.cfg.backends.tor_enabled;

        let backends = vec![
            BackendHealth {
                name: "oxen-node-1".to_string(),
                kind: BackendKind::Oxen,
                latency_ms: 55.0,
                failure_rate: 0.020,
                enabled: oxen_enabled,
            },
            BackendHealth {
                name: "oxen-node-2".to_string(),
                kind: BackendKind::Oxen,
                latency_ms: 70.0,
                failure_rate: 0.040,
                enabled: oxen_enabled,
            },
            BackendHealth {
                name: "tor-exit-1".to_string(),
                kind: BackendKind::Tor,
                latency_ms: 250.0,
                failure_rate: 0.010,
                enabled: tor_enabled,
            },
        ];

        RouterSnapshot { backends }
    }

    /// Return the current snapshot (for `status` command).
    pub fn status(&self) -> Result<RouterSnapshot, Box<dyn std::error::Error>> {
        Ok(self.sample_health())
    }

    /// Choose the best backend for a given target.
    ///
    /// Policy:
    /// - Prefer enabled Oxen nodes with lowest latency.
    /// - If no enabled Oxen nodes, prefer enabled Tor exits with lowest latency.
    /// - If nothing is enabled, return an error.
    pub fn choose_backend(
        &self,
        target: &str,
    ) -> Result<BackendChoice, Box<dyn std::error::Error>> {
        let snapshot = self.sample_health();

        // First: enabled Oxen nodes, sorted by latency.
        let mut oxen_candidates: Vec<&BackendHealth> = snapshot
            .backends
            .iter()
            .filter(|b| b.enabled && matches!(b.kind, BackendKind::Oxen))
            .collect();

        oxen_candidates.sort_by(|a, b| {
            a.latency_ms
                .partial_cmp(&b.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(best_oxen) = oxen_candidates.first() {
            return Ok(BackendChoice {
                target: target.to_string(),
                backend: (*best_oxen).clone(),
            });
        }

        // Fallback: enabled Tor exits, sorted by latency.
        let mut tor_candidates: Vec<&BackendHealth> = snapshot
            .backends
            .iter()
            .filter(|b| b.enabled && matches!(b.kind, BackendKind::Tor))
            .collect();

        tor_candidates.sort_by(|a, b| {
            a.latency_ms
                .partial_cmp(&b.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(best_tor) = tor_candidates.first() {
            return Ok(BackendChoice {
                target: target.to_string(),
                backend: (*best_tor).clone(),
            });
        }

        Err("no enabled backends available".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BackendConfig, GoldDustConfig};

    #[test]
    fn oxen_enabled_prefers_oxen() {
        let cfg = GoldDustConfig {
            backends: BackendConfig {
                oxen_enabled: true,
                tor_enabled: true,
            },
        };

        let router = Router::new(cfg);
        let choice = router.choose_backend("example.com:443").unwrap();

        // With our static values, Oxen should be preferred over Tor.
        assert_eq!(choice.backend.kind, BackendKind::Oxen);
        assert!(choice.backend.latency_ms < 200.0);
    }

    #[test]
    fn disabling_oxen_falls_back_to_tor() {
        let cfg = GoldDustConfig {
            backends: BackendConfig {
                oxen_enabled: false,
                tor_enabled: true,
            },
        };

        let router = Router::new(cfg);
        let choice = router.choose_backend("example.com:443").unwrap();

        // With Oxen disabled, Tor should be selected.
        assert_eq!(choice.backend.kind, BackendKind::Tor);
        assert!(choice.backend.latency_ms > 200.0);
    }

    #[test]
    fn disabling_everything_errors() {
        let cfg = GoldDustConfig {
            backends: BackendConfig {
                oxen_enabled: false,
                tor_enabled: false,
            },
        };

        let router = Router::new(cfg);
        let result = router.choose_backend("example.com:443");

        assert!(result.is_err());
    }
}
