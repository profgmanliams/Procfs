//! Functions related to the in-kernel key management and retention facility
//!
//! For more details on this facility, see the `keyrings(7)` man page.
use crate::{build_internal_error, expect, from_str, ProcResult};
use bitflags::bitflags;
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::BufRead, time::Duration};

bitflags! {
    /// Various key flags
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct KeyFlags: u32 {
            /// The key has been instantiated
            const INSTANTIATED = 0x01;
            /// THe key has been revoked
            const REVOKED = 0x02;
            /// The key is dead
            ///
            /// I.e. the key type has been unregistered.  A key may be briefly in this state during garbage collection.
            const DEAD = 0x04;
            /// The key contributes to the user's quota
            const QUOTA = 0x08;
            /// The key is under construction via a callback to user space
            const UNDER_CONSTRUCTION = 0x10;
            /// The key is negatively instantiated
            const NEGATIVE = 0x20;
            /// The key has been invalidated
            const INVALID = 0x40;
    }
}

bitflags! {
    /// Bitflags that represent the permissions for a key
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct PermissionFlags: u32 {
        /// The attributes of the key may be read
        ///
        /// This includes the type, description, and access rights (excluding the security label)
        const VIEW = 0x01;
        /// For a key: the payload of the key may be read.  For a keyring: the list of serial numbers (keys) to which the keyring has links may be read.
        const READ = 0x02;
        /// The payload of the key may be updated and the key may be revoked.
        ///
        /// For a keyring, links may be added to or removed from the keyring, and the keyring
        /// may be cleared completely (all links are removed).
        const WRITE = 0x04;
        /// The key may be found by a search.
        ///
        /// For keyrings: keys and keyrings that are linked to by the keyring may be searched.
        const SEARCH = 0x08;
        /// Links may be created from keyrings to the key.
        ///
        /// The initial link to a key that is established when the key is created doesn't require this permission.
        const LINK = 0x10;
        /// The ownership details and security label of the key may be changed, the key's expiration
        /// time may be set, and the key may be revoked.
        const SETATTR = 0x20;
        const ALL = Self::VIEW.bits() | Self::READ.bits() | Self::WRITE.bits() | Self::SEARCH.bits() | Self::LINK.bits() | Self::SETATTR.bits();
    }
}

impl KeyFlags {
    fn from_str(s: &str) -> KeyFlags {
        let mut me = KeyFlags::empty();

        let mut chars = s.chars();
        match chars.next() {
            Some(c) if c == 'I' => me.insert(KeyFlags::INSTANTIATED),
            _ => {}
        }
        match chars.next() {
            Some(c) if c == 'R' => me.insert(KeyFlags::REVOKED),
            _ => {}
        }
        match chars.next() {
            Some(c) if c == 'D' => me.insert(KeyFlags::DEAD),
            _ => {}
        }
        match chars.next() {
            Some(c) if c == 'Q' => me.insert(KeyFlags::QUOTA),
            _ => {}
        }
        match chars.next() {
            Some(c) if c == 'U' => me.insert(KeyFlags::UNDER_CONSTRUCTION),
            _ => {}
        }
        match chars.next() {
            Some(c) if c == 'N' => me.insert(KeyFlags::NEGATIVE),
            _ => {}
        }
        match chars.next() {
            Some(c) if c == 'i' => me.insert(KeyFlags::INVALID),
            _ => {}
        }

        me
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Permissions {
    pub possessor: PermissionFlags,
    pub user: PermissionFlags,
    pub group: PermissionFlags,
    pub other: PermissionFlags,
}
impl Permissions {
    fn from_str(s: &str) -> ProcResult<Permissions> {
        let possessor = PermissionFlags::from_bits(from_str!(u32, &s[0..2], 16))
            .ok_or_else(|| build_internal_error!(format!("Unable to parse {:?} as PermissionFlags", s)))?;

        let user = PermissionFlags::from_bits(from_str!(u32, &s[2..4], 16))
            .ok_or_else(|| build_internal_error!(format!("Unable to parse {:?} as PermissionFlags", s)))?;

        let group = PermissionFlags::from_bits(from_str!(u32, &s[4..6], 16))
            .ok_or_else(|| build_internal_error!(format!("Unable to parse {:?} as PermissionFlags", s)))?;

        let other = PermissionFlags::from_bits(from_str!(u32, &s[6..8], 16))
            .ok_or_else(|| build_internal_error!(format!("Unable to parse {:?} as PermissionFlags", s)))?;

        Ok(Permissions {
            possessor,
            user,
            group,
            other,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum KeyTimeout {
    Permanent,
    Expired,
    Timeout(Duration),
}

