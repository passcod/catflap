use nix::sys::socket::{
    bind, getsockname, listen, socket, AddressFamily, InetAddr, SockAddr, SockFlag, SockProtocol,
    SockType,
};
use nix::{errno::Errno, Result};

use std::net::SocketAddr;
use std::os::unix::io::RawFd;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketType {
    Tcp4,
    Udp4,
    Tcp6,
    Udp6,
    Raw4,
    Raw6,
}

impl SocketType {
    pub fn short(self) -> &'static str {
        match self {
            Self::Tcp4 | Self::Tcp6 => "tcp",
            Self::Udp4 | Self::Udp6 => "udp",
            Self::Raw4 => "raw/ipv4",
            Self::Raw6 => "raw/ipv6",
        }
    }

    pub fn with_addr(self, addr: Option<SocketAddr>) -> String {
        match (self, addr) {
            (Self::Raw4, _) | (Self::Raw6, _) => self.short().into(),
            (_, Some(a)) => format!("{}/{}", a, self.short()),
            _ => unreachable!(),
        }
    }
}

pub fn on(addr: SocketAddr, socktype: SocketType) -> Result<RawFd> {
    let sock = match socktype {
        SocketType::Raw4 => custom_socket(
            AddressFamily::Inet,
            SockType::Raw,
            SockFlag::empty(),
            libc::IPPROTO_RAW,
        ),
        SocketType::Raw6 => custom_socket(
            AddressFamily::Inet6,
            SockType::Raw,
            SockFlag::empty(),
            libc::IPPROTO_RAW,
        ),
        SocketType::Tcp4 => socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            Some(SockProtocol::Tcp),
        ),
        SocketType::Tcp6 => socket(
            AddressFamily::Inet6,
            SockType::Stream,
            SockFlag::empty(),
            Some(SockProtocol::Tcp),
        ),
        SocketType::Udp4 => socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            Some(SockProtocol::Udp),
        ),
        SocketType::Udp6 => socket(
            AddressFamily::Inet6,
            SockType::Datagram,
            SockFlag::empty(),
            Some(SockProtocol::Udp),
        ),
    }?;

    let result = Ok(())
        .and_then(|_| {
            if socktype.short() == "raw" {
                Ok(())
            } else {
                bind(sock, &SockAddr::new_inet(InetAddr::from_std(&addr)))
            }
        })
        .and_then(|_| {
            if socktype.short() == "tcp" {
                listen(sock, 1)
            } else {
                Ok(())
            }
        })
        .and_then(|_| Ok(sock));

    if result.is_err() {
        unsafe { libc::close(sock) };
    }

    result
}

fn custom_socket(
    domain: AddressFamily,
    ty: SockType,
    flags: SockFlag,
    protocol: libc::c_int,
) -> Result<RawFd> {
    // adapted from nix source
    let mut ty = ty as libc::c_int;
    ty |= flags.bits();

    let res = unsafe { libc::socket(domain as libc::c_int, ty, protocol) };
    Errno::result(res)
}

pub fn at(sock: RawFd) -> Result<SockAddr> {
    getsockname(sock)
}
