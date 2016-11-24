//! Global kernel info / tuning miscellaneous stuff
//!
//! The files in this directory can be used to tune and monitor miscellaneous
//! and general things in the operation of the Linux kernel.

use std::cmp;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};

use bitflags::bitflags;

use crate::{read_value, write_value, ProcError, ProcResult};

pub mod keys;
pub mod random;

/// Represents a kernel version, in major.minor.release version.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u16) -> Version {
        Version { major, minor, patch }
    }

    /// Returns the kernel version of the currently running kernel.
    ///
    /// This is taken from `/proc/sys/kernel/osrelease`;
    pub fn current() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/osrelease")
    }

    /// Cached version of the kernel version.
    pub(crate) fn cached() -> ProcResult<Self> {
        const SENTINEL: u32 = 0;
        static KERNEL: AtomicU32 = AtomicU32::new(SENTINEL);

        // Try to load the kernel version.
        let mut kernel = KERNEL.load(Ordering::Relaxed);

        // If we haven't loaded the kernel version yet, try to here.
        if kernel == SENTINEL {
            kernel = Self::current()?.to_u32();

            // Try to store it in the cache.
            KERNEL.store(kernel, Ordering::Release);
        }

        Ok(Self::from_u32(kernel))
    }

    /// Convert kernel version to an arbitrary `u32` value.
    fn to_u32(self) -> u32 {
        let [lo, hi] = u16::to_ne_bytes(self.patch);
        u32::from_ne_bytes([self.major, self.minor, lo, hi])
    }

    /// Convert kernel version from an arbitrary `u32` value.
    fn from_u32(val: u32) -> Self {
        let [major, minor, lo, hi] = u32::to_ne_bytes(val);
        Self {
            major,
            minor,
            patch: u16::from_ne_bytes([lo, hi])
        }
    }

    /// Parses a kernel version string, in major.minor.release syntax.
    ///
    /// Note that any extra information (stuff after a dash) is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use procfs::KernelVersion;
    /// let a = KernelVersion::from_str("3.16.0-6-amd64").unwrap();
    /// let b = KernelVersion::new(3, 16, 0);
    /// assert_eq!(a, b);
    ///
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        let pos = s.find(|c: char| c != '.' && !c.is_ascii_digit());
        let kernel = if let Some(pos) = pos {
            let (s, _) = s.split_at(pos);
            s
        } else {
            s
        };
        let mut kernel_split = kernel.split('.');

        let major = kernel_split.next().ok_or("Missing major version component")?;
        let minor = kernel_split.next().ok_or("Missing minor version component")?;
        let patch = kernel_split.next().ok_or("Missing patch version component")?;

        let major = major.parse().map_err(|_| "Failed to parse major version")?;
        let minor = minor.parse().map_err(|_| "Failed to parse minor version")?;
        let patch = patch.parse().map_err(|_| "Failed to parse patch version")?;

        Ok(Version { major, minor, patch })
    }
}

impl FromStr for Version {
    type Err = &'static str;

    /// Parses a kernel version string, in major.minor.release syntax.
    ///
    /// Note that any extra information (stuff after a dash) is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use procfs::KernelVersion;
    /// let a: KernelVersion = "3.16.0-6-amd64".parse().unwrap();
    /// let b = KernelVersion::new(3, 16, 0);
    /// assert_eq!(a, b);
    ///
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::from_str(s)
    }
}

impl cmp::Ord for Version {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.major.cmp(&other.major) {
            cmp::Ordering::Equal => match self.minor.cmp(&other.minor) {
                cmp::Ordering::Equal => self.patch.cmp(&other.patch),
                x => x,
            },
            x => x,
        }
    }
}

impl cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a kernel type
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Type {
    pub sysname: String,
}

impl Type {
    pub fn new(sysname: String) -> Type {
        Type { sysname }
    }

    /// Read the kernel type from current running kernel
    ///
    /// Defined in `include/linux/uts.h` as UTS_SYSNAME, default is "Linux".
    /// The file is located at `/proc/sys/kernel/ostype`.
    pub fn current() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/ostype")
    }
}

impl FromStr for Type {
    type Err = &'static str;

    /// Parse a kernel type string
    ///
    /// Notice that in Linux source code, it is defined as a single string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Type::new(s.to_string()))
    }
}

/// Represents a kernel build information
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuildInfo {
    pub version: String,
    pub flags: HashSet<String>,
    /// This field contains any extra data from the /proc/sys/kernel/version file. It generally contains the build date of the kernel, but the format of the date can vary.
    ///
    /// A method named `extra_date` is provided which would try to parse some date formats. When the date format is not supported, an error will be returned. It depends on chrono feature.
    pub extra: String,
}

impl BuildInfo {
    pub fn new(version: &str, flags: HashSet<String>, extra: String) -> BuildInfo {
        BuildInfo {
            version: version.to_string(),
            flags,
            extra,
        }
    }

    /// Read the kernel build information from current running kernel
    ///
    /// Generated by `scripts/mkcompile_h` when building the kernel.
    /// The file is located at `/proc/sys/kernel/version`.
    pub fn current() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/version")
    }

    /// Check if SMP is ON
    pub fn smp(&self) -> bool {
        self.flags.contains("SMP")
    }

    /// Check if PREEMPT is ON
    pub fn preempt(&self) -> bool {
        self.flags.contains("PREEMPT")
    }

    /// Check if PREEMPTRT is ON
    pub fn preemptrt(&self) -> bool {
        self.flags.contains("PREEMPTRT")
    }

    /// Return version number
    ///
    /// This would parse number from first digits of version string. For example, #21~1 to 21.
    pub fn version_number(&self) -> ProcResult<u32> {
        let mut version_str = String::new();
        for c in self.version.chars() {
            if c.is_ascii_digit() {
                version_str.push(c);
            } else {
                break;
            }
        }
        let version_number: u32 = version_str.parse().map_err(|_| "Failed to parse version number")?;
        Ok(version_number)
    }

    /// Parse extra field to `DateTime` object
    ///
    /// This function may fail as TIMESTAMP can be various formats.
    #[cfg(feature = "chrono")]
    pub fn extra_date(&self) -> ProcResult<chrono::DateTime<chrono::Local>> {
        if let Ok(dt) =
            chrono::DateTime::parse_from_str(&format!("{} +0000", &self.extra), "%a %b %d %H:%M:%S UTC %Y %z")
        {
            return Ok(dt.with_timezone(&chrono::Local));
        }
        if let Ok(dt) = chrono::DateTime::parse_from_str(&self.extra, "%a, %d %b %Y %H:%M:%S %z") {
            return Ok(dt.with_timezone(&chrono::Local));
        }
        Err(ProcError::Other("Failed to parse extra field to date".to_string()))
    }
}

impl FromStr for BuildInfo {
    type Err = &'static str;

    /// Parse a kernel build information string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut version = String::new();
        let mut flags: HashSet<String> = HashSet::new();
        let mut extra: String = String::new();

        let mut splited = s.split(' ');
        let version_str = splited.next();
        if let Some(version_str) = version_str {
            if let Some(stripped) = version_str.strip_prefix('#') {
                version.push_str(stripped);
            } else {
                return Err("Failed to parse kernel build version");
            }
        } else {
            return Err("Failed to parse kernel build version");
        }

        for s in &mut splited {
            if s.chars().all(char::is_uppercase) {
                flags.insert(s.to_string());
            } else {
                extra.push_str(s);
                extra.push(' ');
                break;
            }
        }
        let remains: Vec<&str> = splited.collect();
        extra.push_str(&remains.join(" "));

        Ok(BuildInfo { version, flags, extra })
    }
}

/// Returns the maximum process ID number.
///
/// This is taken from `/proc/sys/kernel/pid_max`.
///
/// # Example
///
/// ```
/// let pid_max = procfs::sys::kernel::pid_max().unwrap();
///
/// let pid = 42; // e.g. from user input, CLI args, etc.
///
/// if pid > pid_max {
///     eprintln!("bad process ID: {}", pid)
/// } else {
///     println!("good process ID: {}", pid);
/// }
/// ```
pub fn pid_max() -> ProcResult<i32> {
    read_value("/proc/sys/kernel/pid_max")
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// Represents the data from `/proc/sys/kernel/sem`
pub struct SemaphoreLimits {
    /// The maximum semaphores per semaphore set
    pub semmsl: u64,
    /// A system-wide limit on the number of semaphores in all semaphore sets
    pub semmns: u64,
    /// The maximum number of operations that may be specified in a semop(2) call
    pub semopm: u64,
    /// A system-wide limit on the maximum number of semaphore identifiers
    pub semmni: u64,
}

impl SemaphoreLimits {
    pub fn new() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/sem")
    }

    fn from_str(s: &str) -> Result<Self, &'static str> {
        let mut s = s.split_ascii_whitespace();

        let semmsl = s.next().ok_or("Missing SEMMSL")?;
        let semmns = s.next().ok_or("Missing SEMMNS")?;
        let semopm = s.next().ok_or("Missing SEMOPM")?;
        let semmni = s.next().ok_or("Missing SEMMNI")?;

        let semmsl = semmsl.parse().map_err(|_| "Failed to parse SEMMSL")?;
        let semmns = semmns.parse().map_err(|_| "Failed to parse SEMMNS")?;
        let semopm = semopm.parse().map_err(|_| "Failed to parse SEMOPM")?;
        let semmni = semmni.parse().map_err(|_| "Failed to parse SEMMNI")?;

        Ok(SemaphoreLimits {
            semmsl,
            semmns,
            semopm,
            semmni,
        })
    }
}

impl FromStr for SemaphoreLimits {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SemaphoreLimits::from_str(s)
    }
}

/// Returns the system-wide limit on the total number of pages of System V shared memory
///
/// This is taken from `/proc/sys/kernel/shmall`
pub fn shmall() -> ProcResult<u64> {
    read_value("/proc/sys/kernel/shmall")
}

/// Returns the limit on the maximum (System V IPC) shared memory segment size that can be created.
/// The value defaults to SHMMAX
///
/// See also [set_shmmax](crate::sys::kernel::set_shmmax)
///
/// This is taken from `/proc/sys/kernel/shmmax`
pub fn shmmax() -> ProcResult<u64> {
    read_value("/proc/sys/kernel/shmmax")
