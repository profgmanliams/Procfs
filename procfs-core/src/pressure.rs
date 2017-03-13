//! Pressure stall information retrieved from `/proc/pressure/cpu`,
//! `/proc/pressure/memory` and `/proc/pressure/io`
//! may not be available on kernels older than 4.20.0
//! For reference: <https://lwn.net/Articles/759781/>
//!
//! See also: <https://www.kernel.org/doc/Documentation/accounting/psi.txt>

use crate::{ProcError, ProcResult};
use std::collections::HashMap;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Pressure stall information for either CPU, memory, or IO.
///
/// See also: <https://www.kernel.org/doc/Documentation/accounting/psi.txt>
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct PressureRecord {
    /// 10 second window
    ///
    /// The percentage of time, over a 10 second window, that either some or all tasks were stalled
    /// waiting for a resource.
    pub avg10: f32,
    /// 60 second window
    ///
    /// The percentage of time, over a 60 second window, that either some or all tasks were stalled
    /// waiting for a resource.
    pub avg60: f32,
    /// 300 second window
    ///
    /// The percentage of time, over a 300 second window, that either some or all tasks were stalled
    /// waiting for a resource.
    pub avg300: f32,
    /// Total stall time (in microseconds).
    pub total: u64,
}

/// CPU pressure information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct CpuPressure {
    /// This record indicates the share of time in which at least some tasks are stalled.
    pub some: PressureRecord,
    /// This record indicates this share of time in which all non-idle tasks are stalled
    /// simultaneously.
    ///
    /// At the system level CPU full is set to zero.
    pub full: PressureRecord,
}

impl super::FromBufRead for CpuPressure {
    fn from_buf_read<R: std::io::BufRead>(mut r: R) -> ProcResult<Self> {
        let (some, full) = get_pressure(r)?;
        Ok(CpuPressure { some, full })
    }
}

/// Memory pressure information
