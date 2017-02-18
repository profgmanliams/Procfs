use super::Process;
use crate::{build_internal_error, ProcResult};
use procfs_core::process::{Namespace, Namespaces};
use rustix::fs::{AtFlags, Mode, OFlags};
use std::{collections::HashMap, ffi::OsString};

impl Process {
    /// Describes namespaces to which the process with the corresponding PID belongs.
    /// Doc reference: <https://man7.org/linux/man-pages/man7/namespaces.7.html>
    /// The namespace type is the key for the HashMap, i.e 'net', 'user', etc.
    pub fn namespaces(&self) -> ProcResult<Namespaces> {
        let mut namespaces = HashMap::new();
        let dir_ns = wrap_io_error!(
