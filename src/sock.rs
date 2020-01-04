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
            SocketType::Tcp4 => "tcp",
            SocketType::Tcp6 => "tcp",
            SocketType::Udp4 => "udp",
            SocketType::Udp6 => "udp",
            SocketType::Raw4 => "raw",
            SocketType::Raw6 => "raw",
        }
    }
}

pub fn on(addr: SocketAddr, socktype: SocketType) -> Result<RawFd> {
    let sock = match socktype {
        SocketType::Raw4 => custom_socket(
            AddressFamily::Inet6,
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
            if socktype.short() != "raw" {
                bind(sock, &SockAddr::new_inet(InetAddr::from_std(&addr)))
            } else {
                Ok(())
            }
        })
        .and_then(|_| {
            if socktype.short() == "tcp" {
                listen(sock, 1)
            } else {
                Ok(())
            }
        })
        .and_then(|_| {
            Ok(sock)
        });

    if !result.is_ok() {
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
