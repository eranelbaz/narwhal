//! Driver registry shared by every host that needs to address a
//! [`narwhal_core::DatabaseDriver`] by name (the TUI app, the MCP
//! server, the headless CLI). Concrete driver implementations are
//! pulled in by cargo features so a build can ship only the engines
//! it needs.
//!
//! Replaces the v1.x `narwhal-driver-registry` crate.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use narwhal_core::{DynDatabaseDriver, Error, Result};

/// Every driver narwhal knows about, compiled in or not.
pub const ALL_DRIVERS: &[&str] = &[
    "postgres",
    "mysql",
    "sqlite",
    "duckdb",
    "clickhouse",
    "mssql",
];

/// Actionable reason `name` can't be used, or `None` if it can.
#[must_use]
pub fn unavailable_hint(registry: &DriverRegistry, name: &str) -> Option<String> {
    if registry.is_disabled(name) {
        Some(format!(
            "`{name}` is disabled — run `narwhal drivers enable {name}`"
        ))
    } else if registry.contains(name) {
        None
    } else if ALL_DRIVERS.contains(&name) {
        Some(format!(
            "`{name}` is not in this build — reinstall with `cargo install narwhaldb --features driver-{name}`"
        ))
    } else {
        Some(format!("unknown driver `{name}`"))
    }
}

#[derive(Default, Clone)]
pub struct DriverRegistry {
    drivers: HashMap<&'static str, Arc<dyn DynDatabaseDriver>>,
    disabled: HashSet<&'static str>,
}

impl fmt::Debug for DriverRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DriverRegistry")
            .field("drivers", &self.drivers.keys().collect::<Vec<_>>())
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl DriverRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<D: DynDatabaseDriver + 'static>(&mut self, driver: D) -> &mut Self {
        self.drivers.insert(driver.name(), Arc::new(driver));
        self
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn DynDatabaseDriver>> {
        if self.disabled.contains(name) {
            return Err(Error::DriverDisabled(name.into()));
        }
        self.drivers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::UnknownDriver(name.into()))
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.drivers.contains_key(name)
    }

    /// Compiled in but switched off via `[drivers] enabled`.
    #[must_use]
    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled.contains(name)
    }

    /// Keep only the drivers named in `enabled`; the rest stay known
    /// as disabled so lookups can say how to turn them back on.
    /// `None` keeps every compiled-in driver.
    #[must_use]
    pub fn with_enabled(mut self, enabled: Option<&[String]>) -> Self {
        let Some(enabled) = enabled.filter(|e| !e.is_empty()) else {
            if enabled.is_some() {
                tracing::warn!("[drivers] enabled is empty; enabling every driver");
            }
            return self;
        };
        for name in enabled {
            if !ALL_DRIVERS.contains(&name.as_str()) {
                tracing::warn!(driver = %name, "unknown driver in [drivers] enabled; ignored");
            }
        }
        let off: Vec<&'static str> = self
            .drivers
            .keys()
            .copied()
            .filter(|name| !enabled.iter().any(|e| e == name))
            .collect();
        for name in off {
            self.drivers.remove(name);
            self.disabled.insert(name);
        }
        self
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.drivers.keys().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.drivers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.drivers.len()
    }

    /// Registry preloaded with every driver compiled into this build via
    /// the matching cargo feature.
    #[must_use]
    pub fn with_defaults() -> Self {
        // `mut` is conditional on at least one driver feature; with the
        // default feature set (no drivers) the variable is never
        // assigned and clippy would warn.
        #[cfg_attr(
            not(any(
                feature = "postgres",
                feature = "sqlite",
                feature = "mysql",
                feature = "duckdb",
                feature = "clickhouse",
                feature = "mssql",
            )),
            allow(unused_mut)
        )]
        let mut registry = Self::new();
        #[cfg(feature = "postgres")]
        registry.register(crate::postgres::PostgresDriver::new());
        #[cfg(feature = "sqlite")]
        registry.register(crate::sqlite::SqliteDriver::new());
        #[cfg(feature = "mysql")]
        registry.register(crate::mysql::MysqlDriver::new());
        #[cfg(feature = "duckdb")]
        registry.register(crate::duckdb::DuckdbDriver::new());
        #[cfg(feature = "clickhouse")]
        registry.register(crate::clickhouse::ClickhouseDriver::new());
        #[cfg(feature = "mssql")]
        registry.register(crate::mssql::MssqlDriver::new());
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_returns_unknown_driver() {
        let registry = DriverRegistry::new();
        assert!(registry.is_empty());
        assert!(registry.get("postgres").is_err());
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn with_defaults_registers_enabled_drivers() {
        let registry = DriverRegistry::with_defaults();
        assert!(registry.contains("sqlite"));
    }

    #[cfg(all(feature = "sqlite", feature = "postgres"))]
    #[test]
    fn with_enabled_disables_the_rest() {
        let registry = DriverRegistry::with_defaults().with_enabled(Some(&["sqlite".into()]));
        assert!(registry.get("sqlite").is_ok());
        assert!(registry.is_disabled("postgres"));
        assert!(matches!(
            registry.get("postgres"),
            Err(Error::DriverDisabled(_))
        ));
        assert!(matches!(registry.get("nope"), Err(Error::UnknownDriver(_))));
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn with_defaults_registers_mssql() {
        let registry = DriverRegistry::with_defaults();
        assert!(registry.contains("mssql"));
    }
}
