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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct MemoryPressure {
    /// This record indicates the share of time in which at least some tasks are stalled.
    pub some: PressureRecord,
    /// This record indicates this share of time in which all non-idle tasks are stalled
    /// simultaneously.
    pub full: PressureRecord,
}

impl super::FromBufRead for MemoryPressure {
    fn from_buf_read<R: std::io::BufRead>(r: R) -> ProcResult<Self> {
        let (some, full) = get_pressure(r)?;
        Ok(MemoryPressure { some, full })
    }
}

/// IO pressure information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct IoPressure {
    /// This record indicates the share of time in which at least some tasks are stalled.
    pub some: PressureRecord,
    /// This record indicates this share of time in which all non-idle tasks are stalled
    /// simultaneously.
    pub full: PressureRecord,
}

impl super::FromBufRead for IoPressure {
    fn from_buf_read<R: std::io::BufRead>(r: R) -> ProcResult<Self> {
        let (some, full) = get_pressure(r)?;
        Ok(IoPressure { some, full })
    }
}

fn get_f32(map: &HashMap<&str, &str>, value: &str) -> ProcResult<f32> {
    map.get(value).map_or_else(
        || Err(ProcError::Incomplete(None)),
        |v| v.parse::<f32>().map_err(|_| ProcError::Incomplete(None)),
    )
}

fn get_total(map: &HashMap<&str, &str>) -> ProcResult<u64> {
    map.get("total").map_or_else(
        || Err(ProcError::Incomplete(None)),
        |v| v.parse::<u64>().map_err(|_| ProcError::Incomplete(None)),
    )
}

/// Parse a single `some`/`full` pressure record in the following format:
///
/// ```text
/// full avg10=2.10 avg60=0.12 avg300=0.00 total=391926
/// ```
///
/// See also [`get_pressure`].
pub fn parse_pressure_record(line: &str) -> ProcResult<PressureRecord> {
    let mut parsed = HashMap::new();

    if !line.starts_with("some") && !line.starts_with("full") {
