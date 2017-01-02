// Don't throw clippy warnings for manual string stripping.
// The suggested fix with `strip_prefix` removes support for Rust 1.33 and 1.38
#![allow(clippy::manual_strip)]

//! Information about the networking layer.
//!
//! This module corresponds to the `/proc/net` directory and contains various information about the
//! networking layer.
//!
//! # Example
//!
//! Here's an example that will print out all of the open and listening TCP sockets, and their
//! corresponding processes, if know.  This mimics the "netstat" utility, but for TCP only.  You
//! can run this example yourself with:
//!
//! > cargo run --example=netstat
//!
//! ```rust
//! # use procfs::process::{FDTarget, Stat};
//! # use std::collections::HashMap;
//! let all_procs = procfs::process::all_processes().unwrap();
//!
//! // build up a map between socket inodes and process stat info:
//! let mut map: HashMap<u64, Stat> = HashMap::new();
//! for p in all_procs {
//!     let Ok(process) = p else {
//!         // process vanished
//!         continue;
//!     };
//!     if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
//!         for fd in fds {
//!             if let FDTarget::Socket(inode) = fd.unwrap().target {
//!                 map.insert(inode, stat.clone());
//!             }
//!         }
//!     }
//! }
//!
//! // get the tcp table
//! let tcp = procfs::net::tcp().unwrap();
//! let tcp6 = procfs::net::tcp6().unwrap();
//! println!("{:<26} {:<26} {:<15} {:<8} {}", "Local address", "Remote address", "State", "Inode", "PID/Program name");
//! for entry in tcp.into_iter().chain(tcp6) {
//!     // find the process (if any) that has an open FD to this entry's inode
//!     let local_address = format!("{}", entry.local_address);
//!     let remote_addr = format!("{}", entry.remote_address);
//!     let state = format!("{:?}", entry.state);
//!     if let Some(stat) = map.get(&entry.inode) {
//!         println!("{:<26} {:<26} {:<15} {:<12} {}/{}", local_address, remote_addr, state, entry.inode, stat.pid, stat.comm);
//!     } else {
//!         // We might not always be able to find the process associated with this socket
//!         println!("{:<26} {:<26} {:<15} {:<12} -", local_address, remote_addr, state, entry.inode);
//!     }
//! }
//! ```
use crate::ProcResult;
use crate::{current_system_info, Current};
pub use procfs_core::net::*;
use procfs_core::FromReadSI;
use std::collections::HashMap;

/// Reads the tcp socket table
///
