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
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut vec = Vec::new();

        // First line is a header we need to skip
        for line in r.lines().skip(1) {
            // Check if there might have been an IO error.
            let line = line?;
            let mut line = line.split_whitespace();
            let ip_address = expect!(Ipv4Addr::from_str(expect!(line.next())));
            let hw = from_str!(u32, &expect!(line.next())[2..], 16);
            let hw = ARPHardware::from_bits_truncate(hw);
            let flags = from_str!(u32, &expect!(line.next())[2..], 16);
            let flags = ARPFlags::from_bits_truncate(flags);

            let mac = expect!(line.next());
            let mut mac: Vec<Result<u8, _>> = mac.split(':').map(|s| Ok(from_str!(u8, s, 16))).collect();

            let mac = if mac.len() == 6 {
                let mac_block_f = mac.pop().unwrap()?;
                let mac_block_e = mac.pop().unwrap()?;
                let mac_block_d = mac.pop().unwrap()?;
                let mac_block_c = mac.pop().unwrap()?;
                let mac_block_b = mac.pop().unwrap()?;
                let mac_block_a = mac.pop().unwrap()?;
                if mac_block_a == 0
                    && mac_block_b == 0
                    && mac_block_c == 0
                    && mac_block_d == 0
                    && mac_block_e == 0
                    && mac_block_f == 0
                {
                    None
                } else {
                    Some([
                        mac_block_a,
                        mac_block_b,
                        mac_block_c,
                        mac_block_d,
                        mac_block_e,
                        mac_block_f,
                    ])
                }
            } else {
                None
            };

            // mask is always "*"
            let _mask = expect!(line.next());
            let dev = expect!(line.next());

            vec.push(ARPEntry {
                ip_address,
                hw_type: hw,
                flags,
                hw_address: mac,
                device: dev.to_string(),
            })
        }

        Ok(ArpEntries(vec))
    }
}

/// General statistics for a network interface/device
///
/// For an example, see the [interface_stats.rs](https://github.com/eminence/procfs/tree/master/procfs/examples)
/// example in the source repo.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct DeviceStatus {
    /// Name of the interface
    pub name: String,
    /// Total bytes received
    pub recv_bytes: u64,
    /// Total packets received
    pub recv_packets: u64,
    /// Bad packets received
    pub recv_errs: u64,
    /// Packets dropped
    pub recv_drop: u64,
    /// Fifo overrun
    pub recv_fifo: u64,
    /// Frame alignment errors
    pub recv_frame: u64,
    /// Number of compressed packets received
    pub recv_compressed: u64,
    /// Number of multicast packets received
    pub recv_multicast: u64,

    /// Total bytes transmitted
    pub sent_bytes: u64,
    /// Total packets transmitted
    pub sent_packets: u64,
    /// Number of transmission errors
    pub sent_errs: u64,
    /// Number of packets dropped during transmission
    pub sent_drop: u64,
    pub sent_fifo: u64,
    /// Number of collisions
    pub sent_colls: u64,
    /// Number of packets not sent due to carrier errors
    pub sent_carrier: u64,
    /// Number of compressed packets transmitted
    pub sent_compressed: u64,
}

impl DeviceStatus {
    fn from_str(s: &str) -> ProcResult<DeviceStatus> {
        let mut split = s.split_whitespace();
        let name: String = expect!(from_iter(&mut split));
        let recv_bytes = expect!(from_iter(&mut split));
        let recv_packets = expect!(from_iter(&mut split));
        let recv_errs = expect!(from_iter(&mut split));
        let recv_drop = expect!(from_iter(&mut split));
        let recv_fifo = expect!(from_iter(&mut split));
        let recv_frame = expect!(from_iter(&mut split));
        let recv_compressed = expect!(from_iter(&mut split));
        let recv_multicast = expect!(from_iter(&mut split));
        let sent_bytes = expect!(from_iter(&mut split));
        let sent_packets = expect!(from_iter(&mut split));
        let sent_errs = expect!(from_iter(&mut split));
        let sent_drop = expect!(from_iter(&mut split));
        let sent_fifo = expect!(from_iter(&mut split));
        let sent_colls = expect!(from_iter(&mut split));
        let sent_carrier = expect!(from_iter(&mut split));
        let sent_compressed = expect!(from_iter(&mut split));

        Ok(DeviceStatus {
            name: name.trim_end_matches(':').to_owned(),
            recv_bytes,
            recv_packets,
            recv_errs,
            recv_drop,
            recv_fifo,
            recv_frame,
            recv_compressed,
            recv_multicast,
            sent_bytes,
            sent_packets,
            sent_errs,
            sent_drop,
            sent_fifo,
            sent_colls,
            sent_carrier,
            sent_compressed,
        })
    }
}

/// Device status information for all network interfaces.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct InterfaceDeviceStatus(pub HashMap<String, DeviceStatus>);

impl super::FromBufRead for InterfaceDeviceStatus {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut map = HashMap::new();
        // the first two lines are headers, so skip them
        for line in r.lines().skip(2) {
            let dev = DeviceStatus::from_str(&line?)?;
            map.insert(dev.name.clone(), dev);
        }

        Ok(InterfaceDeviceStatus(map))
    }
}

/// An entry in the ipv4 route table
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct RouteEntry {
    /// Interface to which packets for this route will be sent
    pub iface: String,
    /// The destination network or destination host
    pub destination: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub flags: u16,
    /// Number of references to this route
    pub refcnt: u16,
    /// Count of lookups for the route
    pub in_use: u16,
    /// The 'distance' to the target (usually counted in hops)
    pub metrics: u32,
    pub mask: Ipv4Addr,
    /// Default maximum transmission unit for TCP connections over this route
    pub mtu: u32,
    /// Default window size for TCP connections over this route
    pub window: u32,
    /// Initial RTT (Round Trip Time)
    pub irtt: u32,
}

/// A set of ipv4 routes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct RouteEntries(pub Vec<RouteEntry>);

impl super::FromBufRead for RouteEntries {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut vec = Vec::new();

        // First line is a header we need to skip
        for line in r.lines().skip(1) {
            // Check if there might have been an IO error.
            let line = line?;
            let mut line = line.split_whitespace();
            // network interface name, e.g. eth0
            let iface = expect!(line.next());
            let destination = from_str!(u32, expect!(line.next()), 16).to_ne_bytes().into();
            let gateway = from_str!(u32, expect!(line.next()), 16).to_ne_bytes().into();
            let flags = from_str!(u16, expect!(line.next()), 16);
            let refcnt = from_str!(u16, expect!(line.next()), 10);
            let in_use = from_str!(u16, expect!(line.next()), 10);
            let metrics = from_str!(u32, expect!(line.next()), 10);
            let mask = from_str!(u32, expect!(line.next()), 16).to_ne_bytes().into();
            let mtu = from_str!(u32, expect!(line.next()), 10);
            let window = from_str!(u32, expect!(line.next()), 10);
            let irtt = from_str!(u32, expect!(line.next()), 10);
            vec.push(RouteEntry {
                iface: iface.to_string(),
                destination,
                gateway,
                flags,
                refcnt,
                in_use,
                metrics,
                mask,
                mtu,
                window,
                irtt,
            });
        }

        Ok(RouteEntries(vec))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
/// The indication of whether this entity is acting as an IP gateway in respect
/// to the forwarding of datagrams received by, but not addressed to, this
/// entity.  IP gateways forward datagrams.  IP hosts do not (except those
/// source-routed via the host).
///
/// Note that for some managed nodes, this object may take on only a subset of
/// the values possible. Accordingly, it is appropriate for an agent to return a
/// `badValue` response if a management station attempts to change this object
/// to an inappropriate value.
pub enum IpForwarding {
    /// Acting as a gateway
    Forwarding = 1,
    /// Not acting as a gateway
    NotForwarding = 2,
}

impl IpForwarding {
    pub fn from_u8(num: u8) -> Option<IpForwarding> {
        match num {
            1 => Some(IpForwarding::Forwarding),
            2 => Some(IpForwarding::NotForwarding),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            IpForwarding::Forwarding => 1,
            IpForwarding::NotForwarding => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
/// The algorithm used to determine the timeout value used for retransmitting
/// unacknowledged octets.
pub enum TcpRtoAlgorithm {
    /// None of the following
    Other = 1,
    /// A constant rto
    Constant = 2,
    /// MIL-STD-1778, [Appendix B](https://datatracker.ietf.org/doc/html/rfc1213#appendix-B)
    Rsre = 3,
    /// Van Jacobson's algorithm
    ///
    /// Reference: Jacobson, V., "Congestion Avoidance and Control", SIGCOMM 1988, Stanford, California.
    Vanj = 4,
}

impl TcpRtoAlgorithm {
    pub fn from_u8(num: u8) -> Option<TcpRtoAlgorithm> {
        match num {
            1 => Some(TcpRtoAlgorithm::Other),
            2 => Some(TcpRtoAlgorithm::Constant),
            3 => Some(TcpRtoAlgorithm::Rsre),
            4 => Some(TcpRtoAlgorithm::Vanj),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            TcpRtoAlgorithm::Other => 1,
            TcpRtoAlgorithm::Constant => 2,
            TcpRtoAlgorithm::Rsre => 3,
            TcpRtoAlgorithm::Vanj => 4,
        }
    }
}

/// This struct holds the data needed for the IP, ICMP, TCP, and UDP management
/// information bases for an SNMP agent.
///
/// For more details, see [RFC1213](https://datatracker.ietf.org/doc/html/rfc1213)
/// and [SNMP counter](https://docs.kernel.org/networking/snmp_counter.html)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Snmp {
    pub ip_forwarding: IpForwarding,
    /// The default value inserted into the Time-To-Live field of the IP header
    /// of datagrams originated at this entity, whenever a TTL value is not
    /// supplied by the transport layer protocol.
    pub ip_default_ttl: u32,
    /// The total number of input datagrams received from interfaces, including
    /// those received in error.
    pub ip_in_receives: u64,
    /// The number of input datagrams discarded due to errors in their IP
    /// headers.
    pub ip_in_hdr_errors: u64,
    /// The number of input datagrams discarded because the IP address in their
    /// IP header's destination field was not a valid address to be received at
    /// this entity.
    pub ip_in_addr_errors: u64,
    /// The number of input datagrams for which this entity was not their final
    /// IP destination, as a result of which an attempt was made to find a
    /// route to forward them to that final destination.
    pub ip_forw_datagrams: u64,
    /// The number of locally-addressed datagrams received successfully but
    /// discarded because of an unknown or unsupported protocol.
    pub ip_in_unknown_protos: u64,
    /// The number of input IP datagrams for which no problems were encountered
    /// to prevent their continued processing, but which were discarded
    /// (e.g., for lack of buffer space).
    pub ip_in_discards: u64,
    /// The total number of input datagrams successfully delivered to IP
    /// user-protocols (including ICMP).
    ///
    /// Note that this counter does not include any datagrams discarded while
    /// awaiting re-assembly.
    pub ip_in_delivers: u64,
    /// The total number of IP datagrams which local IP user-protocols
    /// (including ICMP) supplied to IP in requests for transmission.
    ///
    /// Note that this counter does not include any datagrams counted in
    /// ipForwDatagrams.
    pub ip_out_requests: u64,
    /// The number of output IP datagrams for which no problem was encountered
    /// to prevent their transmission to their destination, but which were
    /// discarded (e.g., for lack of buffer space).
    ///
    /// Note that this counter would include datagrams counted in
    /// `IpForwDatagrams` if any such packets met this (discretionary) discard
    /// criterion.
    pub ip_out_discards: u64,
    /// The number of IP datagrams discarded because no route could be found to
    /// transmit them to their destination.
    ///
    /// Note that this counter includes any packets counted in `IpForwDatagrams`
    /// which meet this `no-route' criterion.
    ///
    /// Note that this includes any datagarms which a host cannot route because
    /// all of its default gateways are down.
    pub ip_out_no_routes: u64,
    /// The maximum number of seconds which received fragments are held while
    /// they are awaiting reassembly at this entity.
    pub ip_reasm_timeout: u64,
    /// The number of IP fragments received which needed to be reassembled at
    /// this entity.
    pub ip_reasm_reqds: u64,
    /// The number of IP datagrams successfully re-assembled.
    pub ip_reasm_oks: u64,
    /// The number of failures detected by the IP re-assembly algorithm
    /// (for whatever reason: timed out, errors, etc).
    ///
    /// Note that this is not necessarily a count of discarded IP fragments
    /// since some algorithms (notably the algorithm in [RFC 815](https://datatracker.ietf.org/doc/html/rfc815))
    /// can lose track of the number of fragments by combining them as they are
    /// received.
    pub ip_reasm_fails: u64,
    /// The number of IP datagrams that have been successfully fragmented at
    /// this entity.
    pub ip_frag_oks: u64,
    /// The number of IP datagrams that have been discarded because they needed
    /// to be fragmented at this entity but could not be, e.g., because their
    /// `Don't Fragment` flag was set.
    pub ip_frag_fails: u64,
    /// The number of IP datagram fragments that have been generated as a result
    /// of fragmentation at this entity.
    pub ip_frag_creates: u64,

    /// The total number of ICMP messages which the entity received.
    ///
    /// Note that this counter includes all those counted by `icmp_in_errors`.
    pub icmp_in_msgs: u64,
    /// The number of ICMP messages which the entity received but determined as
    /// having ICMP-specific errors (bad ICMP checksums, bad length, etc.
    pub icmp_in_errors: u64,
    /// This counter indicates the checksum of the ICMP packet is wrong.
    ///
    /// Non RFC1213 field
    pub icmp_in_csum_errors: u64,
    /// The number of ICMP Destination Unreachable messages received.
    pub icmp_in_dest_unreachs: u64,
    /// The number of ICMP Time Exceeded messages received.
    pub icmp_in_time_excds: u64,
    /// The number of ICMP Parameter Problem messages received.
    pub icmp_in_parm_probs: u64,
    /// The number of ICMP Source Quench messages received.
    pub icmp_in_src_quenchs: u64,
    /// The number of ICMP Redirect messages received.
    pub icmp_in_redirects: u64,
    /// The number of ICMP Echo (request) messages received.
    pub icmp_in_echos: u64,
    /// The number of ICMP Echo Reply messages received.
    pub icmp_in_echo_reps: u64,
    /// The number of ICMP Timestamp (request) messages received.
    pub icmp_in_timestamps: u64,
    /// The number of ICMP Timestamp Reply messages received.
    pub icmp_in_timestamp_reps: u64,
    /// The number of ICMP Address Mask Request messages received.
    pub icmp_in_addr_masks: u64,
    /// The number of ICMP Address Mask Reply messages received.
    pub icmp_in_addr_mask_reps: u64,
