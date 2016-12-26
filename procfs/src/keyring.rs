use super::{Current, ProcResult};
use procfs_core::keyring::*;
use std::collections::HashMap;

impl Current for Keys {
    const PATH: &'static str = "/proc/keys";
}

/// Returns a list of the keys for which the reading thread has **view** permission, providing
/// various information about each key.
pub fn keys() -> ProcResult<Vec<Key>> {
    Keys::current().map(|k| k.0)
}

impl Current for KeyUsers {
    const PATH: &'static str = "/proc/key-users";
