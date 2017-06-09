use crate::{FromStrRadix, ProcResult};
use std::collections::HashMap;
use std::io::BufRead;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Status information about the process, based on the `/proc/<pid>/status` file.
///
/// Not all fields are available in every kernel.  These fields have `Option<T>` types.
/// In general, the current kernel version will tell you what fields you can expect, but this
/// isn't totally reliable, since some kernels might backport certain fields, or fields might
/// only be present if certain kernel configuration options are enabled.  Be prepared to
/// handle `None` values.
///
/// New fields to this struct may be added at any time (even without a major or minor semver bump).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct Status {
    /// Command run by this process.
    pub name: String,
    /// Process umask, expressed in octal with a leading zero; see umask(2).  (Since Linux 4.7.)
    pub umask: Option<u32>,
    /// Current state of the process.
    pub state: String,
    /// Thread group ID (i.e., Process ID).
    pub tgid: i32,
    /// NUMA group ID (0 if none; since Linux 3.13).
    pub ngid: Option<i32>,
    /// Thread ID (see gettid(2)).
    pub pid: i32,
    /// PID of parent process.
    pub ppid: i32,
    /// PID of process tracing this process (0 if not being traced).
    pub tracerpid: i32,
    /// Real UID.
    pub ruid: u32,
    /// Effective UID.
    pub euid: u32,
    /// Saved set UID.
    pub suid: u32,
    /// Filesystem UID.
    pub fuid: u32,
    /// Real GID.
    pub rgid: u32,
    /// Effective GID.
    pub egid: u32,
    /// Saved set GID.
    pub sgid: u32,
    /// Filesystem GID.
    pub fgid: u32,
    /// Number of file descriptor slots currently allocated.
    pub fdsize: u32,
    /// Supplementary group list.
    pub groups: Vec<u32>,
    /// Thread group ID (i.e., PID) in each of the PID
    /// namespaces of which (pid)[struct.Status.html#structfield.pid] is a member.  The leftmost entry
    /// shows the value with respect to the PID namespace of the
    /// reading process, followed by the value in successively
    /// nested inner namespaces.  (Since Linux 4.1.)
    pub nstgid: Option<Vec<i32>>,
    /// Thread ID in each of the PID namespaces of which
    /// (pid)[struct.Status.html#structfield.pid] is a member.  The fields are ordered as for NStgid.
    /// (Since Linux 4.1.)
    pub nspid: Option<Vec<i32>>,
    /// Process group ID in each of the PID namespaces of
    /// which (pid)[struct.Status.html#structfield.pid] is a member.  The fields are ordered as for NStgid.  (Since Linux 4.1.)
    pub nspgid: Option<Vec<i32>>,
    /// NSsid: descendant namespace session ID hierarchy Session ID
    /// in each of the PID namespaces of which (pid)[struct.Status.html#structfield.pid] is a member.
    /// The fields are ordered as for NStgid.  (Since Linux 4.1.)
    pub nssid: Option<Vec<i32>>,
    /// Peak virtual memory size by kibibytes.
    pub vmpeak: Option<u64>,
    /// Virtual memory size by kibibytes.
    pub vmsize: Option<u64>,
    /// Locked memory size by kibibytes (see mlock(3)).
    pub vmlck: Option<u64>,
    /// Pinned memory size by kibibytes (since Linux 3.2).  These are
    /// pages that can't be moved because something needs to
    /// directly access physical memory.
    pub vmpin: Option<u64>,
    /// Peak resident set size by kibibytes ("high water mark").
    pub vmhwm: Option<u64>,
    /// Resident set size by kibibytes.  Note that the value here is the
    /// sum of RssAnon, RssFile, and RssShmem.
    pub vmrss: Option<u64>,
    /// Size of resident anonymous memory by kibibytes.  (since Linux 4.5).
    pub rssanon: Option<u64>,
    /// Size of resident file mappings by kibibytes.  (since Linux 4.5).
    pub rssfile: Option<u64>,
    /// Size of resident shared memory by kibibytes (includes System V
    /// shared memory, mappings from tmpfs(5), and shared anonymous
    /// mappings).  (since Linux 4.5).
    pub rssshmem: Option<u64>,
    /// Size of data by kibibytes.
    pub vmdata: Option<u64>,
    /// Size of stack by kibibytes.
    pub vmstk: Option<u64>,
    /// Size of text seg‐ments by kibibytes.
    pub vmexe: Option<u64>,
    /// Shared library code size by kibibytes.
    pub vmlib: Option<u64>,
    /// Page table entries size by kibibytes (since Linux 2.6.10).
    pub vmpte: Option<u64>,
    /// Swapped-out virtual memory size by anonymous private
    /// pages by kibibytes; shmem swap usage is not included (since Linux 2.6.34).
    pub vmswap: Option<u64>,
    /// Size of hugetlb memory portions by kB.  (since Linux 4.4).
    pub hugetlbpages: Option<u64>,
    /// Number of threads in process containing this thread.
    pub threads: u64,
    /// This field contains two slash-separated numbers that
    /// relate to queued signals for the real user ID of this
    /// process.  The first of these is the number of currently
    /// queued signals for this real user ID, and the second is the
    /// resource limit on the number of queued signals for this
    /// process (see the description of RLIMIT_SIGPENDING in
    /// getrlimit(2)).
    pub sigq: (u64, u64),
    /// Number of signals pending for thread (see pthreads(7) and signal(7)).
    pub sigpnd: u64,
    /// Number of signals pending for process as a whole (see pthreads(7) and signal(7)).
    pub shdpnd: u64,
    /// Masks indicating signals being blocked (see signal(7)).
    pub sigblk: u64,
    /// Masks indicating signals being ignored (see signal(7)).
    pub sigign: u64,
    /// Masks indicating signals being caught (see signal(7)).
    pub sigcgt: u64,
    /// Masks of capabilities enabled in inheritable sets (see capabilities(7)).
    pub capinh: u64,
    /// Masks of capabilities enabled in permitted sets (see capabilities(7)).
    pub capprm: u64,
    /// Masks of capabilities enabled in effective sets (see capabilities(7)).
    pub capeff: u64,
    /// Capability Bounding set (since Linux 2.6.26, see capabilities(7)).
    pub capbnd: Option<u64>,
    /// Ambient capability set (since Linux 4.3, see capabilities(7)).
    pub capamb: Option<u64>,
    /// Value of the no_new_privs bit (since Linux 4.10, see prctl(2)).
    pub nonewprivs: Option<u64>,
    /// Seccomp mode of the process (since Linux 3.8, see
    /// seccomp(2)).  0 means SECCOMP_MODE_DISABLED; 1 means SEC‐
    /// COMP_MODE_STRICT; 2 means SECCOMP_MODE_FILTER.  This field
    /// is provided only if the kernel was built with the CON‐
    /// FIG_SECCOMP kernel configuration option enabled.
    pub seccomp: Option<u32>,
    /// Speculative store bypass mitigation status.
    pub speculation_store_bypass: Option<String>,
    /// Mask of CPUs on which this process may run (since Linux 2.6.24, see cpuset(7)).
    pub cpus_allowed: Option<Vec<u32>>,
    /// Same as previous, but in "list format" (since Linux 2.6.26, see cpuset(7)).
    pub cpus_allowed_list: Option<Vec<(u32, u32)>>,
    /// Mask of memory nodes allowed to this process (since Linux 2.6.24, see cpuset(7)).
    pub mems_allowed: Option<Vec<u32>>,
    /// Same as previous, but in "list format" (since Linux 2.6.26, see cpuset(7)).
    pub mems_allowed_list: Option<Vec<(u32, u32)>>,
    /// Number of voluntary context switches (since Linux 2.6.23).
    pub voluntary_ctxt_switches: Option<u64>,
    /// Number of involuntary context switches (since Linux 2.6.23).
    pub nonvoluntary_ctxt_switches: Option<u64>,

    /// Contains true if the process is currently dumping core.
    ///
    /// This information can be used by a monitoring process to avoid killing a processing that is
    /// currently dumping core, which could result in a corrupted core dump file.
    ///
    /// (Since Linux 4.15)
    pub core_dumping: Option<bool>,

    /// Contains true if the process is allowed to use THP
    ///
    /// (Since Linux 5.0)
    pub thp_enabled: Option<bool>,
}

impl crate::FromBufRead for Status {
    fn from_buf_read<R: BufRead>(reader: R) -> ProcResult<Self> {
        let mut map = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            let mut s = line.split(':');
            let field = expect!(s.next());
            let value = expect!(s.next()).trim();

            map.insert(field.to_string(), value.to_string());
        }

        let status = Status {
            name: expect!(map.remove("Name")),
            umask: map.remove("Umask").map(|x| Ok(from_str!(u32, &x, 8))).transpose()?,
            state: expect!(map.remove("State")),
            tgid: from_str!(i32, &expect!(map.remove("Tgid"))),
            ngid: map.remove("Ngid").map(|x| Ok(from_str!(i32, &x))).transpose()?,
            pid: from_str!(i32, &expect!(map.remove("Pid"))),
            ppid: from_str!(i32, &expect!(map.remove("PPid"))),
            tracerpid: from_str!(i32, &expect!(map.remove("TracerPid"))),
            ruid: expect!(Status::parse_uid_gid(expect!(map.get("Uid")), 0)),
