use std::{collections::HashMap, io::BufRead};

use super::ProcResult;
use std::str::FromStr;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// A mountpoint entry under `/proc/mounts`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[allow(non_snake_case)]
pub struct MountEntry {
    /// Device
    pub fs_spec: String,
    /// Mountpoint
    pub fs_file: String,
    /// FS type
    pub fs_vfstype: String,
    /// Mount options
    pub fs_mntops: HashMap<String, Option<String>>,
    /// Dump
    pub fs_freq: u8,
    /// Check
    pub fs_passno: u8,
}

impl super::FromBufRead for Vec<MountEntry> {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut vec = Vec::new();

        for line in r.lines() {
            let line = expect!(line);

            // The format is `fs_spec fs_file fs_vfstype fs_mntops fs_freq fs_passno`.
            // Each field is normally space-mangled (e.g. `\040` for a literal space),
            // so a plain split on ' ' would yield six fields. In practice some
            // drivers emit `fs_mntops` with unescaped spaces inside option values
            // (notably the 9p mount Docker Desktop on WSL2 creates, where the
            // option string contains `path=C:\Program Files\...`). To stay robust
            // to that, take the first three fields from the front and the last two
            // from the back; whatever is left in between is `fs_mntops`.
            let mut head = line.splitn(4, ' ');
            let fs_spec = unmangle_octal(expect!(head.next()));
            let fs_file = unmangle_octal(expect!(head.next()));
            let fs_vfstype = unmangle_octal(expect!(head.next()));
            let rest = expect!(head.next());

            let mut tail = rest.rsplitn(3, ' ');
