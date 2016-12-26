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
            /// this flag is included, binfmt_misc will add an argument to the argument vector for this purpose, thus
            /// preserving the original `argv[0]`.
            ///
            /// For example, If your interp is set to `/bin/foo` and you run `blah` (which is in `/usr/local/bin`),
            /// then the kernel will execute `/bin/foo` with `argv[]` set to `["/bin/foo", "/usr/local/bin/blah", "blah"]`.
            ///
            /// The interp has to be aware of this so it can execute `/usr/local/bin/blah` with `argv[]` set to `["blah"]`.
            const P = 0x01;

            /// Open Binary
            ///
            /// Legacy behavior of binfmt_misc is to pass the full path
            /// of the binary to the interpreter as an argument. When this flag is
            /// included, binfmt_misc will open the file for reading and pass its
            /// descriptor as an argument, instead of the full path, thus allowing
            /// the interpreter to execute non-readable binaries. This feature
            /// should be used with care - the interpreter has to be trusted not to
            //// emit the contents of the non-readable binary.
            const O = 0x02;

            /// Credentials
            ///
            ///  Currently, the behavior of binfmt_misc is to calculate
            /// the credentials and security token of the new process according to
            /// the interpreter. When this flag is included, these attributes are
            /// calculated according to the binary. It also implies the `O` flag.
            /// This feature should be used with care as the interpreter
            /// will run with root permissions when a setuid binary owned by root
