use nix::sys::socket::{
    bind, getsockname, listen, socket, AddressFamily, InetAddr, SockAddr, SockFlag, SockProtocol,
    SockType,
};

use std::net::SocketAddr;
use std::os::unix::io::RawFd;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketType {
    Tcp4,
    Udp4,
    Tcp6,
    Udp6,
}

impl SocketType {
    pub fn short(self) -> &'static str {
        match self {
            Self::Tcp4 | Self::Tcp6 => "tcp",
            Self::Udp4 | Self::Udp6 => "udp",
        }
    }

    pub fn with_addr(self, addr: Option<SocketAddr>) -> String {
        match addr {
            None => self.short().into(),
            Some(a) => format!("{}/{}", a, self.short()),
        }
    }
}

pub fn on(addr: SocketAddr, socktype: SocketType) -> nix::Result<RawFd> {
    let sock = match socktype {
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
            bind(sock, &SockAddr::new_inet(InetAddr::from_std(&addr)))
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

pub fn at(sock: RawFd) -> nix::Result<SockAddr> {
    getsockname(sock)
}
