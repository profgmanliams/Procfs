use crate::from_iter;
use crate::ProcResult;
use std::io::Read;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Provides scheduler statistics of the process, based on the `/proc/<pid>/schedstat` file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Schedstat {
    /// Time spent on the cpu.
    ///
    /// Measured in nanoseconds.
    pub sum_exec_runtime: u64,
    /// Time spent waiting on a runqueue.
    ///
    /// Measured in nanoseconds.
    pub run_delay: u64,
    /// \# of timeslices run on this cpu.
    pub pcount: u64,
