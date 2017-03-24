use crate::{ProcError, ProcResult};

use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Process limits
///
/// For more details about each of these limits, see the `getrlimit` man page.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Limits {
    /// Max Cpu Time
    ///
    /// This is a limit, in seconds, on the amount of CPU time that the process can consume.
    pub max_cpu_time: Limit,

    /// Max file size
    ///
    /// This is the maximum size in bytes of files that the process may create.
    pub max_file_size: Limit,

    /// Max data size
    ///
    /// This is the maximum size of the process's data segment (initialized data, uninitialized
    /// data, and heap).
    pub max_data_size: Limit,

    /// Max stack size
    ///
    /// This is the maximum size of the process stack, in bytes.
    pub max_stack_size: Limit,

    /// Max core file size
    ///
    /// This is the maximum size of a *core* file in bytes that the process may dump.
    pub max_core_file_size: Limit,

    /// Max resident set
    ///
    /// This is a limit (in bytes) on the process's resident set (the number of virtual pages
    /// resident in RAM).
    pub max_resident_set: Limit,

    /// Max processes
    ///
    /// This is a limit on the number of extant process (or, more precisely on Linux, threads) for
    /// the real user rID of the calling process.
    pub max_processes: Limit,

    /// Max open files
    ///
    /// This specifies a value one greater than the maximum file descriptor number that can be
    /// opened by this process.
    pub max_open_files: Limit,

    /// Max locked memory
    ///
    /// This is the maximum number of bytes of memory that may be locked into RAM.
    pub max_locked_memory: Limit,

    /// Max address space
    ///
    /// This is the maximum size of the process's virtual memory (address space).
    pub max_address_space: Limit,

    /// Max file locks
    ///
    /// This is a limit on the combined number of flock locks and fcntl leases that this process
    /// may establish.
    pub max_file_locks: Limit,

    /// Max pending signals
    ///
    /// This is a limit on the number of signals that may be queued for the real user rID of the
    /// calling process.
    pub max_pending_signals: Limit,

    /// Max msgqueue size
    ///
    /// This is a limit on the number of bytes that can be allocated for POSIX message queues for
    /// the real user rID of the calling process.
    pub max_msgqueue_size: Limit,

    /// Max nice priority
    ///
    /// This specifies a ceiling to which the process's nice value can be raised using
    /// `setpriority` or `nice`.
    pub max_nice_priority: Limit,

    /// Max realtime priority
    ///
    /// This specifies a ceiling on the real-time priority that may be set for this process using
    /// `sched_setscheduler` and `sched_setparam`.
    pub max_realtime_priority: Limit,
