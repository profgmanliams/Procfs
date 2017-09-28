use super::ProcState;
use super::StatFlags;
use crate::{from_iter, from_iter_optional, ProcResult};
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

use std::io::Read;
use std::str::FromStr;

/// Status information about the process, based on the `/proc/<pid>/stat` file.
///
/// Not all fields are available in every kernel.  These fields have `Option<T>` types.
///
/// New fields to this struct may be added at any time (even without a major or minor semver bump).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct Stat {
    /// The process ID.
    pub pid: i32,
    /// The filename of the executable, without the parentheses.
    ///
    /// This is visible whether or not the executable is swapped out.
    ///
    /// Note that if the actual comm field contains invalid UTF-8 characters, they will be replaced
    /// here by the U+FFFD replacement character.
    pub comm: String,
    /// Process State.
    ///
    /// See [state()](#method.state) to get the process state as an enum.
    pub state: char,
    /// The PID of the parent of this process.
    pub ppid: i32,
    /// The process group ID of the process.
    pub pgrp: i32,
    /// The session ID of the process.
    pub session: i32,
    /// The controlling terminal of the process.
    ///
    /// The minor device number is contained in the combination of bits 31 to 20 and  7  to  0;
    /// the major device number is in bits 15 to 8.
    ///
    /// See [tty_nr()](#method.tty_nr) to get this value decoded into a (major, minor) tuple
    pub tty_nr: i32,
    /// The ID of the foreground process group of the controlling terminal of the process.
    pub tpgid: i32,
    /// The kernel flags  word of the process.
    ///
    /// For bit meanings, see the PF_* defines in  the  Linux  kernel  source  file
    /// [`include/linux/sched.h`](https://github.com/torvalds/linux/blob/master/include/linux/sched.h).
    ///
    /// See [flags()](#method.flags) to get a [`StatFlags`](struct.StatFlags.html) bitfield object.
    pub flags: u32,
    /// The number of minor faults the process has made which have not required loading a memory
    /// page from disk.
    pub minflt: u64,
    /// The number of minor faults that the process's waited-for children have made.
    pub cminflt: u64,
    /// The number of major faults the process has made which have required loading a memory page
    /// from disk.
    pub majflt: u64,
    /// The number of major faults that the process's waited-for children have made.
    pub cmajflt: u64,
    /// Amount of time that this process has been scheduled in user mode, measured in clock ticks
    /// (divide by `ticks_per_second()`).
    ///
    /// This includes guest time, guest_time (time spent running a virtual CPU, see below), so that
    /// applications that are not aware of the guest time field  do not lose that time from their
    /// calculations.
    pub utime: u64,
    /// Amount of time that this process has been scheduled in kernel mode, measured in clock ticks
    /// (divide by `ticks_per_second()`).
    pub stime: u64,

    /// Amount  of  time  that  this  process's  waited-for  children  have  been  scheduled  in
    /// user  mode,  measured  in clock ticks (divide by `ticks_per_second()`).
    ///
    /// This includes guest time, cguest_time (time spent running a virtual CPU, see below).
    pub cutime: i64,

    /// Amount of time that this process's waited-for  children  have  been  scheduled  in  kernel
    /// mode,  measured  in  clock  ticks  (divide  by `ticks_per_second()`).
    pub cstime: i64,
    /// For processes running a real-time scheduling policy (policy below; see sched_setscheduler(2)),
    /// this is the negated scheduling priority, minus one;
    ///
    /// That is, a number in the range -2 to -100,
    /// corresponding to real-time priority 1 to 99.  For processes running under a non-real-time
    /// scheduling policy, this is the raw nice value (setpriority(2)) as represented in the kernel.
    /// The kernel stores nice values as numbers in the range 0 (high) to 39  (low),  corresponding
    /// to the user-visible nice range of -20 to 19.
    /// (This explanation is for Linux 2.6)
    ///
    /// Before Linux 2.6, this was a scaled value based on the scheduler weighting given to this process.
    pub priority: i64,
    /// The nice value (see `setpriority(2)`), a value in the range 19 (low priority) to -20 (high priority).
    pub nice: i64,
    /// Number  of  threads in this process (since Linux 2.6).  Before kernel 2.6, this field was
    /// hard coded to 0 as a placeholder for an earlier removed field.
    pub num_threads: i64,
    /// The time in jiffies before the next SIGALRM is sent to the process due to an interval
    /// timer.
    ///
    /// Since kernel 2.6.17, this  field is no longer maintained, and is hard coded as 0.
    pub itrealvalue: i64,
    /// The time the process started after system boot.
    ///
    /// In kernels before Linux 2.6, this value was expressed in  jiffies.  Since  Linux 2.6, the
