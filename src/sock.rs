use libc::close;

use nix::sys::socket::{
    bind, getsockname, listen, socket, AddressFamily, InetAddr, SockAddr, SockFlag, SockProtocol,
    SockType,
};
use nix::Result;

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

pub fn on(addr: SocketAddr, socktype: SocketType) -> Result<RawFd> {
    let sock = match socktype {
        SocketType::Tcp4 => socket(
            AddressFamily::Inet,
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
        SocketType::Raw4 => socket(AddressFamily::Inet, SockType::Raw, SockFlag::empty(), None),
        SocketType::Tcp6 => socket(
            AddressFamily::Inet6,
            SockType::Stream,
            SockFlag::empty(),
            Some(SockProtocol::Tcp),
        ),
        SocketType::Udp6 => socket(
            AddressFamily::Inet6,
            SockType::Datagram,
            SockFlag::empty(),
            Some(SockProtocol::Udp),
        ),
        SocketType::Raw6 => socket(AddressFamily::Inet6, SockType::Raw, SockFlag::empty(), None),
    }?;

    let result = Ok(())
        .and_then(|_| bind(sock, &SockAddr::new_inet(InetAddr::from_std(&addr))))
        .and_then(|_| listen(sock, 1))
        .and_then(|_| Ok(sock));

    if !result.is_ok() {
        unsafe { close(sock) };
    }

    result
}

pub fn at(sock: RawFd) -> Result<SockAddr> {
    getsockname(sock)
}
