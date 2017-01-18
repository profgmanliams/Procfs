use super::{Current, ProcResult};
use procfs_core::keyring::*;
use std::collections::HashMap;

impl Current for Keys {
    const PATH: &'static str = "/proc/keys";
}

