use std::io::Read;
use std::path::{Path, PathBuf};

use super::{FileWrapper, Io, Schedstat, Stat, Status, Syscall};
use crate::{ProcError, ProcResult};
use procfs_core::FromRead;
use rustix::fd::{BorrowedFd, OwnedFd};

/// A task (aka Thread) inside of a [`Process`](crate::process::Process)
///
/// Created by [`Process::tasks`](crate::process::Process::tasks), tasks in
/// general are similar to Processes and should have mostly the same fields.
#[derive(Debug)]
pub struct Task {
    fd: OwnedFd,
    /// The ID of the process that this task belongs to
    pub pid: i32,
    /// The task ID
    pub tid: i32,
    /// Task root: `/proc/<pid>/task/<tid>`
    pub(crate) root: PathBuf,
}

impl Task {
    /// Create a new `Task` inside of the process
    ///
    /// This API is designed to be ergonomic from inside of [`TasksIter`](super::TasksIter)
    pub(crate) fn from_process_at<P: AsRef<Path>, Q: AsRef<Path>>(
        base: P,
        dirfd: BorrowedFd,
        path: Q,
        pid: i32,
        tid: i32,
    ) -> ProcResult<Task> {
        use rustix::fs::{Mode, OFlags};

        let p = path.as_ref();
        let root = base.as_ref().join(p);
        let fd = wrap_io_error!(
            root,
            rustix::fs::openat(
                dirfd,
                p,
                OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty()
            )
        )?;

        Ok(Task { fd, pid, tid, root })
    }

    /// Thread info from `/proc/<pid>/task/<tid>/stat`
    ///
    /// Many of the returned fields will be the same as the parent process, but some fields like `utime` and `stime` will be per-task
    pub fn stat(&self) -> ProcResult<Stat> {
        self.read("stat")
    }

    /// Thread info from `/proc/<pid>/task/<tid>/status`
    ///
    /// Many of the returned fields will be the same as the parent process
    pub fn status(&self) -> ProcResult<Status> {
        self.read("status")
    }

    /// Thread IO info from `/proc/<pid>/task/<tid>/io`
    ///
    /// This data will be unique per task.
    pub fn io(&self) -> ProcResult<Io> {
        self.read("io")
    }

    /// Thread scheduler info from `/proc/<pid>/task/<tid>/schedstat`
    ///
    /// This data will be unique per task.
    pub fn schedstat(&self) -> ProcResult<Schedstat> {
        self.read("schedstat")
    }

    /// Returns the status info from `/proc/<pid>/task/<tid>/syscall`.
    pub fn syscall(&self) -> ProcResult<Syscall> {
        self.read("syscall")
    }

    /// Thread children from `/proc/<pid>/task/<tid>/children`
    ///
    /// WARNING:
    /// This interface is not reliable unless all the child processes are stopped or frozen.
    /// If a child task exits while the file is being read, non-exiting children may be omitted.
    /// See the procfs(5) man page for more information.
    ///
    /// This data will be unique per task.
    pub fn children(&self) -> ProcResult<Vec<u32>> {
        let mut buf = String::new();
        let mut file = FileWrapper::open_at(&self.root, &self.fd, "children")?;
        file.read_to_string(&mut buf)?;
        buf.split_whitespace()
            .map(|child| {
                child
                    .parse()
                    .map_err(|_| ProcError::Other("Failed to parse task's child PIDs".to_string()))
            })
            .collect()
    }

    /// Deliberately generate an IO error
    #[cfg(test)]
    pub(crate) fn generate_error(&self) -> ProcResult<()> {
        let _ = FileWrapper::open_at(&self.root, &self.fd, "does_not_exist")?;
        Ok(())
    }

    /// Parse a file relative to the task proc structure.
    pub fn read<P, T>(&self, path: P) -> ProcResult<T>
    where
        P: AsRef<Path>,
        T: FromRead,
    {
        FromRead::from_read(FileWrapper::open_at(&self.root, &self.fd, path)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::process::{Io, Syscall};
    use crate::ProcError;
    use rustix;
    use std::process;
    use std::sync::{Arc, Barrier};

    #[test]
    #[cfg(not(tarpaulin))] // this test is unstable under tarpaulin, and i'm yet sure why
                           // When this test runs in CI, run it single-threaded
    fn test_task_runsinglethread() {
        use std::io::Read;

        let me = crate::process::Process::myself().unwrap();

        let (work_barrier, w_a, w_b) = {
            let b = Arc::new(Barrier::new(3));
            (b.clone(), b.clone(), b)
        };
