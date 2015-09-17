use bitflags::bitflags;
use std::path::Path;

use crate::{build_internal_error, from_str, read_value, ProcResult};

/// Returns true if the miscellaneous Binary Formats system is enabled.
pub fn enabled() -> ProcResult<bool> {
    let val: String = read_value("/proc/sys/fs/binfmt_misc/status")?;
    Ok(val == "enabled")
}

fn hex_to_vec(hex: &str) -> ProcResult<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(build_internal_error!(format!(
            "Hex string {:?} has non-even length",
            hex
        )));
    }
    let mut idx = 0;
    let mut data = Vec::new();
    while idx < hex.len() {
        let byte = from_str!(u8, &hex[idx..idx + 2], 16);
        data.push(byte);
        idx += 2;
    }

    Ok(data)
}

#[derive(Debug, Clone)]
pub enum BinFmtData {
    /// A BinFmt entry based on a file extension (does not include the period)
    Extension(String),
    /// A BinFmt entry based on magic string matching
    Magic { offset: u8, magic: Vec<u8>, mask: Vec<u8> },
}

/// A registered binary format entry
///
/// For more info, see the kernel doc Documentation/admin-guide/binfmt-misc.rst
#[derive(Debug, Clone)]
pub struct BinFmtEntry {
    /// The name of the entry
    ///
    /// Corresponds to a file in /proc/sys/fs/binfmt_misc/
    pub name: String,
    /// Is the entry enabled or not
    pub enabled: bool,
    /// Full path to the interpreter to run this entry
    pub interpreter: String,
    ///
    pub flags: BinFmtFlags,
    pub data: BinFmtData,
}

impl BinFmtEntry {
    pub(crate) fn from_string(name: String, data: &str) -> ProcResult<Self> {
        let mut enabled = false;
        let mut interpreter = String::new();

        let mut ext = None;

        let mut offset = 0;
        let mut magic = Vec::new();
        let mut mask = Vec::new();
        let mut flags = BinFmtFlags::empty();

        for line in data.lines() {
            if line == "enabled" {
                enabled = true;
            } else if let Some(stripped) = line.strip_prefix("interpreter ") {
                interpreter = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("flags:") {
                flags = BinFmtFlags::from_str(stripped);
            } else if let Some(stripped) = line.strip_prefix("extension .") {
                ext = Some(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix("offset ") {
                offset = from_str!(u8, stripped);
            } else if let Some(stripped) = line.strip_prefix("magic ") {
                let hex = stripped;
                magic = hex_to_vec(dbg!(hex))?;
            } else if let Some(stripped) = line.strip_prefix("mask ") {
                let hex = stripped;
                mask = hex_to_vec(hex)?;
            }
        }

        if !magic.is_empty() && mask.is_empty() {
            mask.resize(magic.len(), 0xff);
        }

        Ok(BinFmtEntry {
            name,
            enabled,
            interpreter,
            flags,
            data: if let Some(ext) = ext {
                BinFmtData::Extension(ext)
            } else {
                BinFmtData::Magic { magic, mask, offset }
            },
        })
    }
}

bitflags! {
    /// Various key flags
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct BinFmtFlags: u8 {
            /// Preserve Argv[0]
            ///
            /// Legacy behavior of binfmt_misc is to overwrite the original argv[0] with the full path to the binary. When
