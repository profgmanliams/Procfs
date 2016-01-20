use crate::{expect, from_str, ProcResult};
use std::io::BufRead;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// The type of a file lock
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum LockType {
    /// A BSD file lock created using `flock`
    FLock,

    /// A POSIX byte-range lock created with `fcntl`
    Posix,

    /// An Open File Description (ODF) lock created with `fnctl`
    ODF,

    /// Some other unknown lock type
    Other(String),
}

impl LockType {
    pub fn as_str(&self) -> &str {
        match self {
            LockType::FLock => "FLOCK",
            LockType::Posix => "POSIX",
            LockType::ODF => "ODF",
            LockType::Other(s) => s.as_ref(),
        }
    }
}

impl From<&str> for LockType {
    fn from(s: &str) -> LockType {
        match s {
            "FLOCK" => LockType::FLock,
            "OFDLCK" => LockType::ODF,
            "POSIX" => LockType::Posix,
            x => LockType::Other(x.to_string()),
        }
    }
}

/// The mode of a lock (advisory or mandatory)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum LockMode {
    Advisory,
    Mandatory,

    /// Some other unknown lock mode
    Other(String),
}

impl LockMode {
    pub fn as_str(&self) -> &str {
        match self {
            LockMode::Advisory => "ADVISORY",
            LockMode::Mandatory => "MANDATORY",
            LockMode::Other(s) => s.as_ref(),
        }
    }
}

impl From<&str> for LockMode {
    fn from(s: &str) -> LockMode {
        match s {
            "ADVISORY" => LockMode::Advisory,
            "MANDATORY" => LockMode::Mandatory,
            x => LockMode::Other(x.to_string()),
        }
    }
}

/// The kind of a lock (read or write)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum LockKind {
    /// A read lock (or BSD shared lock)
    Read,

    /// A write lock (or a BSD exclusive lock)
    Write,

    /// Some other unknown lock kind
    Other(String),
}

impl LockKind {
    pub fn as_str(&self) -> &str {
        match self {
            LockKind::Read => "READ",
            LockKind::Write => "WRITE",
            LockKind::Other(s) => s.as_ref(),
        }
    }
}

impl From<&str> for LockKind {
    fn from(s: &str) -> LockKind {
        match s {
            "READ" => LockKind::Read,
            "WRITE" => LockKind::Write,
            x => LockKind::Other(x.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
/// Details about an individual file lock
///
/// For an example, see the [lslocks.rs](https://github.com/eminence/procfs/tree/master/procfs/examples)
/// example in the source repo.
pub struct Lock {
    /// The type of lock
