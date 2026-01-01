//! Common prelude for browser-core crate
//! 
//! This module provides standard imports and utilities for consistent
//! error handling and logging across the crate.

pub use anyhow::{Context, Result, bail, ensure};
pub use thiserror::Error;
pub use tracing::{debug, error, info, trace, warn, instrument};

/// Extension trait for Option types to provide better error messages
pub trait OptionExt<T> {
    /// Converts Option to Result with a descriptive error message
    fn ok_or_err(self, msg: &str) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_err(self, msg: &str) -> Result<T> {
        self.ok_or_else(|| anyhow::anyhow!("{}", msg))
    }
}

/// Extension trait for Result types to add context
pub trait ResultExt<T, E> {
    /// Adds context to an error
    fn with_ctx(self, msg: &str) -> Result<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T, E> for std::result::Result<T, E> {
    fn with_ctx(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
    }
}

/// Get current Unix timestamp in seconds
/// Returns 0 if system time is before Unix epoch (should never happen)
pub fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Get current Unix timestamp in milliseconds
pub fn unix_timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
