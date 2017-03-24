use bitflags::bitflags;

use crate::{from_iter, ProcResult};

use std::collections::HashMap;
use std::io::{BufRead, Lines};
use std::path::PathBuf;
use std::time::Duration;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct NFSServerCaps: u32 {

        const NFS_CAP_READDIRPLUS = 1;
        const NFS_CAP_HARDLINKS = (1 << 1);
        const NFS_CAP_SYMLINKS = (1 << 2);
        const NFS_CAP_ACLS = (1 << 3);
        const NFS_CAP_ATOMIC_OPEN = (1 << 4);
        const NFS_CAP_LGOPEN = (1 << 5);
        const NFS_CAP_FILEID = (1 << 6);
        const NFS_CAP_MODE = (1 << 7);
        const NFS_CAP_NLINK = (1 << 8);
        const NFS_CAP_OWNER = (1 << 9);
        const NFS_CAP_OWNER_GROUP = (1 << 10);
        const NFS_CAP_ATIME = (1 << 11);
        const NFS_CAP_CTIME = (1 << 12);
        const NFS_CAP_MTIME = (1 << 13);
        const NFS_CAP_POSIX_LOCK = (1 << 14);
        const NFS_CAP_UIDGID_NOMAP = (1 << 15);
        const NFS_CAP_STATEID_NFSV41 = (1 << 16);
        const NFS_CAP_ATOMIC_OPEN_V1 = (1 << 17);
        const NFS_CAP_SECURITY_LABEL = (1 << 18);
        const NFS_CAP_SEEK = (1 << 19);
        const NFS_CAP_ALLOCATE = (1 << 20);
        const NFS_CAP_DEALLOCATE = (1 << 21);
        const NFS_CAP_LAYOUTSTATS = (1 << 22);
        const NFS_CAP_CLONE = (1 << 23);
        const NFS_CAP_COPY = (1 << 24);
        const NFS_CAP_OFFLOAD_CANCEL = (1 << 25);
    }
}

/// Information about a all mounts in a process's mount namespace.
///
/// This data is taken from the `/proc/[pid]/mountinfo` file.
pub struct MountInfos(pub Vec<MountInfo>);

impl MountInfos {
    /// Returns an borrowed iterator.
    pub fn iter(&self) -> std::slice::Iter<'_, MountInfo> {
        self.into_iter()
    }
}

impl crate::FromBufRead for MountInfos {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let lines = r.lines();
        let mut vec = Vec::new();
        for line in lines {
            vec.push(MountInfo::from_line(&line?)?);
        }

        Ok(MountInfos(vec))
    }
}

impl IntoIterator for MountInfos {
    type IntoIter = std::vec::IntoIter<MountInfo>;
    type Item = MountInfo;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a MountInfos {
    type IntoIter = std::slice::Iter<'a, MountInfo>;
    type Item = &'a MountInfo;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Information about a specific mount in a process's mount namespace.
///
/// This data is taken from the `/proc/[pid]/mountinfo` file.
///
/// For an example, see the
/// [mountinfo.rs](https://github.com/eminence/procfs/tree/master/procfs/examples) example in the
/// source repo.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct MountInfo {
    /// Mount ID.  A unique ID for the mount (but may be reused after `unmount`)
    pub mnt_id: i32,
    /// Parent mount ID.  The ID of the parent mount (or of self for the root of the mount
    /// namespace's mount tree).
    ///
    /// If the parent mount point lies outside the process's root directory, the ID shown here
    /// won't have a corresponding record in mountinfo whose mount ID matches this parent mount
    /// ID (because mount points that lie outside the process's root directory are not shown in
    /// mountinfo).  As a special case of this point, the process's root mount point may have a
    /// parent mount (for the initramfs filesystem) that lies outside the process's root
    /// directory, and an entry for  that mount point will not appear in mountinfo.
    pub pid: i32,
    /// The value of `st_dev` for files on this filesystem
    pub majmin: String,
    /// The pathname of the directory in the filesystem which forms the root of this mount.
    pub root: String,
    /// The pathname of the mount point relative to the process's root directory.
    pub mount_point: PathBuf,
    /// Per-mount options
    pub mount_options: HashMap<String, Option<String>>,
    /// Optional fields
    pub opt_fields: Vec<MountOptFields>,
    /// Filesystem type
    pub fs_type: String,
    /// Mount source
    pub mount_source: Option<String>,
    /// Per-superblock options.
    pub super_options: HashMap<String, Option<String>>,
}

impl MountInfo {
    pub fn from_line(line: &str) -> ProcResult<MountInfo> {
        let mut split = line.split(' '); // not using split_whitespace because we might have empty fields

        let mnt_id = expect!(from_iter(&mut split));
        let pid = expect!(from_iter(&mut split));
        let majmin: String = expect!(from_iter(&mut split));
        let root = expect!(from_iter(&mut split));
        let mount_point = expect!(from_iter(&mut split));
        let mount_options = {
            let mut map = HashMap::new();
            let all_opts = expect!(split.next());
            for opt in all_opts.split(',') {
                let mut s = opt.splitn(2, '=');
                let opt_name = expect!(s.next());
                map.insert(opt_name.to_owned(), s.next().map(|s| s.to_owned()));
            }
            map
        };

        let mut opt_fields = Vec::new();
        loop {
            let f = expect!(split.next());
            if f == "-" {
                break;
            }
            let mut s = f.split(':');
            let opt = match expect!(s.next()) {
                "shared" => {
                    let val = expect!(from_iter(&mut s));
                    MountOptFields::Shared(val)
                }
                "master" => {
                    let val = expect!(from_iter(&mut s));
                    MountOptFields::Master(val)
                }
                "propagate_from" => {
                    let val = expect!(from_iter(&mut s));
                    MountOptFields::PropagateFrom(val)
                }
                "unbindable" => MountOptFields::Unbindable,
                _ => continue,
            };
            opt_fields.push(opt);
        }
        let fs_type: String = expect!(from_iter(&mut split));
        let mount_source = match expect!(split.next()) {
            "none" | "" => None,
            x => Some(x.to_owned()),
        };
        let super_options = {
            let mut map = HashMap::new();
            let all_opts = expect!(split.next());
            for opt in all_opts.split(',') {
                let mut s = opt.splitn(2, '=');
                let opt_name = expect!(s.next());
                map.insert(opt_name.to_owned(), s.next().map(|s| s.to_owned()));
            }
            map
        };

        Ok(MountInfo {
            mnt_id,
            pid,
            majmin,
            root,
            mount_point,
            mount_options,
            opt_fields,
            fs_type,
            mount_source,
            super_options,
        })
    }
}

/// Optional fields used in [MountInfo]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum MountOptFields {
    /// This mount point is shared in peer group.  Each peer group has a unique ID that is
    /// automatically generated by the kernel, and all mount points in the same peer group will
    /// show the same ID
    Shared(u32),
    /// THis mount is a slave to the specified shared peer group.
    Master(u32),
    /// This mount is a slave and receives propagation from the shared peer group
    PropagateFrom(u32),
    /// This is an unbindable mount
    Unbindable,
}

/// A single entry in [MountStats].
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
