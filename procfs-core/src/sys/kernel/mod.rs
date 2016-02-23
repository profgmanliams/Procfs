//! Global kernel info / tuning miscellaneous stuff
//!
//! The files in this directory can be used to tune and monitor miscellaneous
//! and general things in the operation of the Linux kernel.

use std::cmp;
use std::collections::HashSet;
use std::str::FromStr;

use bitflags::bitflags;

use crate::{ProcError, ProcResult};

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

    /// Parses a kernel version string, in major.minor.release syntax.
    ///
    /// Note that any extra information (stuff after a dash) is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use procfs_core::KernelVersion;
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
    /// # use procfs_core::KernelVersion;
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
