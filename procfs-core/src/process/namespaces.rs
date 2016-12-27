use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Information about a namespace
#[derive(Debug, Clone, Eq)]
#[cfg_attr(all(feature = "serde1", not(target_family = "wasm")), derive(Serialize, Deserialize))]
pub struct Namespace {
    /// Namespace type
    pub ns_type: OsString,
    /// Handle to the namespace
    pub path: PathBuf,
    /// Namespace identifier (inode number)
    pub identifier: u64,
    /// Device id of the namespace
    pub device_id: u64,
