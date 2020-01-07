#![forbid(clippy::pedantic)]
#![deny(clippy::needless_pass_by_value)]
#![allow(
    clippy::cognitive_complexity,
    clippy::shadow_unrelated,
    clippy::similar_names
)]

use nix::sys::socket::SockAddr;
use std::net::SocketAddr;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::str::FromStr;

mod args;
mod sock;

use sock::SocketType;

fn parse_socket_v4(s: &str, free_port: &mut u16) -> Result<SocketAddr, String> {
    use std::net::{Ipv4Addr, SocketAddrV4};

    if s == "auto" || s == ":" {
        let port = *free_port;
        *free_port += 1;
        Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)))
    } else if s.starts_with(':') {
        let port: String = s.chars().skip(1).collect();
        let port = u16::from_str(&port).map_err(|e| e.to_string())?;
        Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)))
    } else {
        SocketAddrV4::from_str(s)
            .or_else(|_| {
                let port = *free_port;
                *free_port += 1;
                SocketAddrV4::from_str(&format!("{}:{}", s, port))
            })
            .map_err(|e| e.to_string())
            .map(|addr| addr.into())
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn check_socket_v4(s: String) -> Result<(), String> {
    parse_socket_v4(&s, &mut 0).map(|_| ())
}

fn parse_socket_v6(s: &str, free_port: &mut u16) -> Result<SocketAddr, String> {
    use std::net::{Ipv6Addr, SocketAddrV6};

    if s == "auto" || s == ":" {
        let port = *free_port;
        *free_port += 1;
        Ok(SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::LOCALHOST,
            port,
            0,
            0,
        )))
    } else if s.starts_with(':') {
        let port: String = s.chars().skip(1).collect();
        let port = u16::from_str(&port).map_err(|e| e.to_string())?;
        Ok(SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::LOCALHOST,
            port,
            0,
            0,
        )))
    } else {
        SocketAddrV6::from_str(s)
            .or_else(|_| {
                let port = *free_port;
                *free_port += 1;
                SocketAddrV6::from_str(&format!("{}:{}", s, port))
            })
            .map_err(|e| e.to_string())
            .map(|addr| addr.into())
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn check_socket_v6(s: String) -> Result<(), String> {
    parse_socket_v6(&s, &mut 0).map(|_| ())
}

fn main() -> Result<(), String> {
    let args = args::parse();

    macro_rules! extract_sockets_of_type {
        ($args:ident, $sockets:ident, $option:literal, $socktype:expr, $parser:ident) => {
            extract_sockets_of_type!($args, $sockets, $option, $socktype, $parser, 5000)
        };
        ($args:ident, $sockets:ident, $option:literal, $socktype:expr, $parser:ident, $start_port:expr) => {
            let mut free_port = $start_port;
            let mut n = $args.occurrences_of($option);
            if n > 0 {
                for s in $args.values_of($option).unwrap() {
                    n = n.saturating_sub(1);
                    $sockets.push((
                        $parser(s, &mut free_port).map_err(|e| e.to_string())?,
                        $socktype,
                    ));
                }

                if n > 0 {
                    for _ in 0..n {
                        $sockets.push((
                            $parser("auto", &mut free_port).map_err(|e| e.to_string())?,
                            $socktype,
                        ));
                    }
                }
            }
        };
    }

    let mut sockets = Vec::new();

    extract_sockets_of_type!(args, sockets, "tcp", SocketType::Tcp4, parse_socket_v4);
    extract_sockets_of_type!(args, sockets, "tcp6", SocketType::Tcp6, parse_socket_v6);
    extract_sockets_of_type!(args, sockets, "udp", SocketType::Udp4, parse_socket_v4);
    extract_sockets_of_type!(args, sockets, "udp6", SocketType::Udp6, parse_socket_v6);

    if sockets.is_empty() {
        sockets.push((parse_socket_v4("auto", &mut 5000)?, SocketType::Tcp4));
    }

    let mut fds = Vec::with_capacity(sockets.len());
    for (addr, socktype) in sockets {
        let fd = sock::on(addr, socktype)
            .map_err(|e| format!("binding {}: {}", socktype.with_addr(Some(addr)), e))?;
        let at = sock::at(fd)
            .map_err(|e| format!("naming {}: {}", socktype.with_addr(Some(addr)), e))?;
        fds.push((fd, at, socktype));
    }

    let env = args.value_of("env").unwrap();
    let nenv = args.value_of("nenv").unwrap();

    println!(
        "[Catflap listening at {}]",
        fds.iter()
            .map(|(fd, at, ty)| format!(
                "{} ({})",
                ty.with_addr(if let SockAddr::Inet(ref a) = at {
                    Some(a.to_std())
                } else {
                    None
                }),
                fd
            ))
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut cmd_args = args.values_of("command").unwrap().collect::<Vec<&str>>();
    let cmd = cmd_args.remove(0);
    let mut cmd = Command::new(cmd);
    cmd.args(cmd_args);

    if env != "-" {
        cmd.env(
            env,
            fds.iter()
                .map(|(fd, _, _)| fd.to_string())
                .collect::<Vec<String>>()
                .join(","),
        );
    }

    if nenv != "-" {
        cmd.env(nenv, fds.len().to_string());
    }

    let err = cmd.exec();
    Err(format!("Error running command: {}", err))
}
