//! Functions related to the in-kernel key management and retention facility
//!
//! For more details on this facility, see the `keyrings(7)` man page.
//!
//! Additional functions can be found in the [keyring](crate::keyring) module.
use crate::{read_value, write_value, ProcResult};

/// GC Delay
///
/// The value in this file specifies the interval, in seconds,
/// after which revoked and expired keys will be garbage collected.
/// The purpose of having such an interval is so that
/// there is a window of time where user space can see an error
/// (respectively EKEYREVOKED and EKEYEXPIRED) that indicates what
/// happened to the key.
///
/// The default value in this file is 300 (i.e., 5 minutes).
///
/// (since Linux 2.6.32)
pub fn gc_delay() -> ProcResult<u32> {
    read_value("/proc/sys/kernel/keys/gc_delay")
}

/// Persistent Keyring Expiry
///
/// This file defines an interval, in seconds, to which the persistent
/// keyring's expiration timer is reset each time the
/// keyring is accessed (via keyctl_get_persistent(3) or the
/// keyctl(2) KEYCTL_GET_PERSISTENT operation.)
///
/// The default value in this file is 259200 (i.e., 3 days).
///
/// (Since Linux 3.13)
pub fn persistent_keyring_expiry() -> ProcResult<u32> {
    read_value("/proc/sys/kernel/keys/persistent_keyring_expiry")
}

/// Max bytes
///
/// This is the maximum number of bytes of data that a nonroot
/// user can hold in the payloads of the keys owned by the user.
///
/// The default value in this file is 20,000.
///
/// (since linux 2.6.26)
pub fn maxbytes() -> ProcResult<u32> {
    read_value("/proc/sys/kernel/keys/maxbytes")
}

/// Set max bytes
