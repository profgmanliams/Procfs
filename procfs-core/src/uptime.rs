use crate::{expect, ProcResult};

use std::io::Read;
use std::str::FromStr;
use std::time::Duration;

/// The uptime of the system, based on the `/proc/uptime` file.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Uptime {
    /// The uptime of the system (including time spent in suspend).
    pub uptime: f64,

    /// The sum of how much time each core has spent idle.
    pub idle: f64,
}

impl super::FromRead for Uptime {
    fn from_read<R: Read>(mut r: R) -> ProcResult<Self> {
        let mut buf = Vec::with_capacity(128);
        r.read_to_end(&mut buf)?;

