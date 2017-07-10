// Don't throw clippy warnings for manual string stripping.
// The suggested fix with `strip_prefix` removes support for Rust 1.33 and 1.38
#![allow(clippy::manual_strip)]

//! Information about the networking layer.
//!
//! This module corresponds to the `/proc/net` directory and contains various information about the
//! networking layer.
use crate::ProcResult;
use crate::{build_internal_error, expect, from_iter, from_str};
use std::collections::HashMap;

use bitflags::bitflags;
use std::io::BufRead;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::{path::PathBuf, str::FromStr};

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum TcpState {
    Established = 1,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    NewSynRecv,
}

impl TcpState {
    pub fn from_u8(num: u8) -> Option<TcpState> {
        match num {
            0x01 => Some(TcpState::Established),
            0x02 => Some(TcpState::SynSent),
            0x03 => Some(TcpState::SynRecv),
            0x04 => Some(TcpState::FinWait1),
            0x05 => Some(TcpState::FinWait2),
            0x06 => Some(TcpState::TimeWait),
            0x07 => Some(TcpState::Close),
            0x08 => Some(TcpState::CloseWait),
            0x09 => Some(TcpState::LastAck),
            0x0A => Some(TcpState::Listen),
            0x0B => Some(TcpState::Closing),
            0x0C => Some(TcpState::NewSynRecv),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            TcpState::Established => 0x01,
            TcpState::SynSent => 0x02,
            TcpState::SynRecv => 0x03,
            TcpState::FinWait1 => 0x04,
            TcpState::FinWait2 => 0x05,
            TcpState::TimeWait => 0x06,
            TcpState::Close => 0x07,
            TcpState::CloseWait => 0x08,
            TcpState::LastAck => 0x09,
            TcpState::Listen => 0x0A,
            TcpState::Closing => 0x0B,
            TcpState::NewSynRecv => 0x0C,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum UdpState {
    Established = 1,
    Close = 7,
}

impl UdpState {
    pub fn from_u8(num: u8) -> Option<UdpState> {
        match num {
            0x01 => Some(UdpState::Established),
            0x07 => Some(UdpState::Close),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            UdpState::Established => 0x01,
            UdpState::Close => 0x07,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum UnixState {
    UNCONNECTED = 1,
    CONNECTING = 2,
    CONNECTED = 3,
    DISCONNECTING = 4,
}

impl UnixState {
    pub fn from_u8(num: u8) -> Option<UnixState> {
        match num {
            0x01 => Some(UnixState::UNCONNECTED),
            0x02 => Some(UnixState::CONNECTING),
            0x03 => Some(UnixState::CONNECTED),
            0x04 => Some(UnixState::DISCONNECTING),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            UnixState::UNCONNECTED => 0x01,
            UnixState::CONNECTING => 0x02,
            UnixState::CONNECTED => 0x03,
            UnixState::DISCONNECTING => 0x04,
        }
    }
}

/// An entry in the TCP socket table
#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct TcpNetEntry {
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub state: TcpState,
    pub rx_queue: u32,
    pub tx_queue: u32,
    pub uid: u32,
    pub inode: u64,
}

/// An entry in the UDP socket table
#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct UdpNetEntry {
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub state: UdpState,
    pub rx_queue: u32,
    pub tx_queue: u32,
    pub uid: u32,
    pub inode: u64,
}

/// An entry in the Unix socket table
#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct UnixNetEntry {
    /// The number of users of the socket
    pub ref_count: u32,
    /// The socket type.
    ///
    /// Possible values are `SOCK_STREAM`, `SOCK_DGRAM`, or `SOCK_SEQPACKET`.  These constants can
    /// be found in the libc crate.
    pub socket_type: u16,
    /// The state of the socket
    pub state: UnixState,
    /// The inode number of the socket
    pub inode: u64,
    /// The bound pathname (if any) of the socket.
    ///
    /// Sockets in the abstract namespace are included, and are shown with a path that commences
    /// with the '@' character.
    pub path: Option<PathBuf>,
}

/// Parses an address in the form 00010203:1234
///
/// Also supports IPv6
fn parse_addressport_str(s: &str, little_endian: bool) -> ProcResult<SocketAddr> {
    let mut las = s.split(':');
    let ip_part = expect!(las.next(), "ip_part");
    let port = expect!(las.next(), "port");
    let port = from_str!(u16, port, 16);

    use std::convert::TryInto;

    let read_u32 = if little_endian {
        u32::from_le_bytes
    } else {
        u32::from_be_bytes
    };

    if ip_part.len() == 8 {
        let bytes = expect!(hex::decode(ip_part));
        let ip_u32 = read_u32(bytes[..4].try_into().unwrap());

        let ip = Ipv4Addr::from(ip_u32);

        Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
    } else if ip_part.len() == 32 {
        let bytes = expect!(hex::decode(ip_part));

        let ip_a = read_u32(bytes[0..4].try_into().unwrap());
        let ip_b = read_u32(bytes[4..8].try_into().unwrap());
        let ip_c = read_u32(bytes[8..12].try_into().unwrap());
        let ip_d = read_u32(bytes[12..16].try_into().unwrap());

        let ip = Ipv6Addr::new(
            ((ip_a >> 16) & 0xffff) as u16,
            (ip_a & 0xffff) as u16,
            ((ip_b >> 16) & 0xffff) as u16,
            (ip_b & 0xffff) as u16,
            ((ip_c >> 16) & 0xffff) as u16,
            (ip_c & 0xffff) as u16,
            ((ip_d >> 16) & 0xffff) as u16,
            (ip_d & 0xffff) as u16,
        );

        Ok(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)))
    } else {
        Err(build_internal_error!(format!(
            "Unable to parse {:?} as an address:port",
            s
        )))
    }
}

/// TCP socket entries.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct TcpNetEntries(pub Vec<TcpNetEntry>);

impl super::FromBufReadSI for TcpNetEntries {
    fn from_buf_read<R: BufRead>(r: R, system_info: &crate::SystemInfo) -> ProcResult<Self> {
        let mut vec = Vec::new();

        // first line is a header we need to skip
        for line in r.lines().skip(1) {
            let line = line?;
            let mut s = line.split_whitespace();
            s.next();
            let local_address = expect!(s.next(), "tcp::local_address");
            let rem_address = expect!(s.next(), "tcp::rem_address");
            let state = expect!(s.next(), "tcp::st");
            let mut tx_rx_queue = expect!(s.next(), "tcp::tx_queue:rx_queue").splitn(2, ':');
            let tx_queue = from_str!(u32, expect!(tx_rx_queue.next(), "tcp::tx_queue"), 16);
            let rx_queue = from_str!(u32, expect!(tx_rx_queue.next(), "tcp::rx_queue"), 16);
            s.next(); // skip tr and tm->when
            s.next(); // skip retrnsmt
            let uid = from_str!(u32, expect!(s.next(), "tcp::uid"));
            s.next(); // skip timeout
            let inode = expect!(s.next(), "tcp::inode");

            vec.push(TcpNetEntry {
                local_address: parse_addressport_str(local_address, system_info.is_little_endian())?,
                remote_address: parse_addressport_str(rem_address, system_info.is_little_endian())?,
                rx_queue,
                tx_queue,
                state: expect!(TcpState::from_u8(from_str!(u8, state, 16))),
                uid,
                inode: from_str!(u64, inode),
            });
        }

        Ok(TcpNetEntries(vec))
    }
}

/// UDP socket entries.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct UdpNetEntries(pub Vec<UdpNetEntry>);

impl super::FromBufReadSI for UdpNetEntries {
    fn from_buf_read<R: BufRead>(r: R, system_info: &crate::SystemInfo) -> ProcResult<Self> {
        let mut vec = Vec::new();

        // first line is a header we need to skip
        for line in r.lines().skip(1) {
            let line = line?;
            let mut s = line.split_whitespace();
            s.next();
            let local_address = expect!(s.next(), "udp::local_address");
            let rem_address = expect!(s.next(), "udp::rem_address");
            let state = expect!(s.next(), "udp::st");
            let mut tx_rx_queue = expect!(s.next(), "udp::tx_queue:rx_queue").splitn(2, ':');
            let tx_queue: u32 = from_str!(u32, expect!(tx_rx_queue.next(), "udp::tx_queue"), 16);
            let rx_queue: u32 = from_str!(u32, expect!(tx_rx_queue.next(), "udp::rx_queue"), 16);
            s.next(); // skip tr and tm->when
            s.next(); // skip retrnsmt
            let uid = from_str!(u32, expect!(s.next(), "udp::uid"));
            s.next(); // skip timeout
            let inode = expect!(s.next(), "udp::inode");

            vec.push(UdpNetEntry {
                local_address: parse_addressport_str(local_address, system_info.is_little_endian())?,
                remote_address: parse_addressport_str(rem_address, system_info.is_little_endian())?,
                rx_queue,
                tx_queue,
                state: expect!(UdpState::from_u8(from_str!(u8, state, 16))),
                uid,
                inode: from_str!(u64, inode),
            });
        }

        Ok(UdpNetEntries(vec))
    }
}

/// Unix socket entries.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct UnixNetEntries(pub Vec<UnixNetEntry>);

impl super::FromBufRead for UnixNetEntries {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut vec = Vec::new();

        // first line is a header we need to skip
        for line in r.lines().skip(1) {
            let line = line?;
            let mut s = line.split_whitespace();
            s.next(); // skip table slot number
            let ref_count = from_str!(u32, expect!(s.next()), 16);
            s.next(); // skip protocol, always zero
            s.next(); // skip internal kernel flags
            let socket_type = from_str!(u16, expect!(s.next()), 16);
            let state = from_str!(u8, expect!(s.next()), 16);
            let inode = from_str!(u64, expect!(s.next()));
            let path = s.next().map(PathBuf::from);

            vec.push(UnixNetEntry {
                ref_count,
                socket_type,
                inode,
                state: expect!(UnixState::from_u8(state)),
                path,
            });
        }

        Ok(UnixNetEntries(vec))
    }
}

/// An entry in the ARP table
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct ARPEntry {
    /// IPv4 address
    pub ip_address: Ipv4Addr,
    /// Hardware type
    ///
    /// This will almost always be ETHER (or maybe INFINIBAND)
    pub hw_type: ARPHardware,
    /// Internal kernel flags
    pub flags: ARPFlags,
    /// MAC Address
    pub hw_address: Option<[u8; 6]>,
    /// Device name
    pub device: String,
}

bitflags! {
    /// Hardware type for an ARP table entry.
    // source: include/uapi/linux/if_arp.h
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct ARPHardware: u32 {
        /// NET/ROM pseudo
        const NETROM = 0;
        /// Ethernet
        const ETHER = 1;
        /// Experimental ethernet
        const EETHER = 2;
        /// AX.25 Level 2
        const AX25 = 3;
        /// PROnet token ring
        const PRONET = 4;
        /// Chaosnet
        const CHAOS = 5;
        /// IEEE 802.2 Ethernet/TR/TB
        const IEEE802 = 6;
        /// Arcnet
        const ARCNET = 7;
        /// APPLEtalk
        const APPLETLK = 8;
        /// Frame Relay DLCI
        const DLCI = 15;
        /// ATM
        const ATM = 19;
        /// Metricom STRIP
        const METRICOM = 23;
        //// IEEE 1394 IPv4 - RFC 2734
        const IEEE1394 = 24;
        /// EUI-64
        const EUI64 = 27;
        /// InfiniBand
        const INFINIBAND = 32;
    }
}

bitflags! {
    /// Flags for ARP entries
    // source: include/uapi/linux/if_arp.h
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct ARPFlags: u32 {
            /// Completed entry
            const COM = 0x02;
            /// Permanent entry
            const PERM = 0x04;
            /// Publish entry
            const PUBL = 0x08;
            /// Has requested trailers
            const USETRAILERS = 0x10;
            /// Want to use a netmask (only for proxy entries)
            const NETMASK = 0x20;
            // Don't answer this address
            const DONTPUB = 0x40;
    }
}

/// ARP table entries.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct ArpEntries(pub Vec<ARPEntry>);

impl super::FromBufRead for ArpEntries {
