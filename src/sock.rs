use nix::sys::socket::{
    bind, getsockname, listen, socket, AddressFamily, InetAddr, SockAddr, SockFlag, SockProtocol,
    SockType,
};

use std::fmt;
use std::net::SocketAddr;
use std::os::unix::io::RawFd;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    pub proto: Protocol,
    pub ip: SocketAddr,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", match self.proto {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        }, self.ip)
    }
}

pub fn on(addr: Address) -> nix::Result<RawFd> {
    let sock = match addr {
        Address {
            proto: Protocol::Tcp,
            ip: SocketAddr::V4(_),
        } => socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            Some(SockProtocol::Tcp),
        ),
        Address {
            proto: Protocol::Tcp,
            ip: SocketAddr::V6(_),
        } => socket(
            AddressFamily::Inet6,
            SockType::Stream,
            SockFlag::empty(),
            Some(SockProtocol::Tcp),
        ),
        Address {
            proto: Protocol::Udp,
            ip: SocketAddr::V4(_),
        } => socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            Some(SockProtocol::Udp),
        ),
        Address {
            proto: Protocol::Udp,
            ip: SocketAddr::V6(_),
        } => socket(
            AddressFamily::Inet6,
            SockType::Datagram,
            SockFlag::empty(),
            Some(SockProtocol::Udp),
        ),
    }?;

    let result = Ok(())
        .and_then(|_| bind(sock, &SockAddr::new_inet(InetAddr::from_std(&addr.ip))))
        .and_then(|_| {
            if let Protocol::Tcp = addr.proto {
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

pub fn at_port(sock: RawFd) -> nix::Result<Option<u16>> {
    getsockname(sock).map(|at| match at {
        SockAddr::Inet(inat) => Some(inat.port()),
        _ => None
    })
}
