extern crate clap;
extern crate libc;
extern crate nix;

use std::net::SocketAddr;
use std::os::unix::process::CommandExt;
use std::process::{exit, Command};
use std::str::FromStr;

mod args;
mod sock;

use sock::SocketType;

fn parse_socket_v4(s: &str, free_port: &mut u16) -> Result<SocketAddr, String> {
    use std::net::{SocketAddrV4, Ipv4Addr};

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

pub fn check_socket_v4(s: String) -> Result<(), String> {
    parse_socket_v4(&s, &mut 0).map(|_| ())
}

fn parse_socket_v6(s: &str, free_port: &mut u16) -> Result<SocketAddr, String> {
    use std::net::{SocketAddrV6, Ipv6Addr};

    if s == "auto" || s == ":" {
        let port = *free_port;
        *free_port += 1;
        Ok(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, port, 0, 0)))
    } else if s.starts_with(':') {
        let port: String = s.chars().skip(1).collect();
        let port = u16::from_str(&port).map_err(|e| e.to_string())?;
        Ok(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, port, 0, 0)))
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

pub fn check_socket_v6(s: String) -> Result<(), String> {
    parse_socket_v6(&s, &mut 0).map(|_| ())
}

fn main() {
    let args = args::parse();

    let mut sockets = Vec::new();

    dbg!(&args);

    let mut free_port = 5000;
    let mut raws = args.occurrences_of("raw");
    if raws > 0 {
        for s in args.values_of("raw").unwrap() {
            raws = raws.saturating_sub(1);
            sockets.push((parse_socket_v4(s, &mut free_port).unwrap(), SocketType::Raw4));
        }

        if raws > 0 {
            for _ in 0..raws {
                sockets.push((parse_socket_v4("auto", &mut free_port).unwrap(), SocketType::Raw4));
            }
        }
    }

    let mut free_port = 5000;
    let mut raw6s = args.occurrences_of("raw6");
    if raw6s > 0 {
        for s in args.values_of("raw6").unwrap() {
            raw6s = raw6s.saturating_sub(1);
            sockets.push((parse_socket_v6(s, &mut free_port).unwrap(), SocketType::Raw6));
        }

        if raw6s > 0 {
            for _ in 0..raw6s {
                sockets.push((parse_socket_v6("auto", &mut free_port).unwrap(), SocketType::Raw6));
            }
        }
    }

    let mut free_port = 5000;
    let mut tcps = args.occurrences_of("tcp");
    if tcps > 0 {
        for s in args.values_of("tcp").unwrap() {
            tcps = tcps.saturating_sub(1);
            sockets.push((parse_socket_v4(s, &mut free_port).unwrap(), SocketType::Tcp4));
        }

        if tcps > 0 {
            for _ in 0..tcps {
                sockets.push((parse_socket_v4("auto", &mut free_port).unwrap(), SocketType::Tcp4));
            }
        }
    }

    let mut free_port = 5000;
    let mut tcp6s = args.occurrences_of("tcp6");
    if tcp6s > 0 {
        for s in args.values_of("tcp6").unwrap() {
            tcp6s = tcp6s.saturating_sub(1);
            sockets.push((parse_socket_v6(s, &mut free_port).unwrap(), SocketType::Tcp6));
        }

        if tcp6s > 0 {
            for _ in 0..tcp6s {
                sockets.push((parse_socket_v6("auto", &mut free_port).unwrap(), SocketType::Tcp6));
            }
        }
    }

    let mut free_port = 5000;
    let mut udps = args.occurrences_of("udp");
    if udps > 0 {
        for s in args.values_of("udp").unwrap() {
            udps = udps.saturating_sub(1);
            sockets.push((parse_socket_v4(s, &mut free_port).unwrap(), SocketType::Udp4));
        }

        if udps > 0 {
            for _ in 0..udps {
                sockets.push((parse_socket_v4("auto", &mut free_port).unwrap(), SocketType::Udp4));
            }
        }
    }

    let mut free_port = 5000;
    let mut udp6s = args.occurrences_of("udp6");
    if udp6s > 0 {
        for s in args.values_of("udp6").unwrap() {
            udp6s = udp6s.saturating_sub(1);
            sockets.push((parse_socket_v6(s, &mut free_port).unwrap(), SocketType::Udp6));
        }

        if udp6s > 0 {
            for _ in 0..udp6s {
                sockets.push((parse_socket_v6("auto", &mut free_port).unwrap(), SocketType::Udp6));
            }
        }
    }

    if sockets.is_empty() {
        sockets.push((parse_socket_v4("auto", &mut free_port).unwrap(), SocketType::Tcp4));
    }

    dbg!(sockets);

    let addr = SocketAddr::from_str(&format!(
        "{}:{}",
        args.value_of("host").unwrap(),
        args.value_of("port").unwrap()
    ))
    .unwrap_or_else(|e| {
        println!("Bad address: {}", e);
        exit(1);
    });

    let socktype = SocketType::Tcp4;

    let fd = sock::on(addr, socktype).unwrap_or_else(|e| {
        println!("Could not bind: {}", e);
        exit(1);
    });

    let at = sock::at(fd).unwrap_or_else(|e| {
        println!("Could not get address: {}", e);
        exit(1);
    });

    let env = args.value_of("env").unwrap();

    println!("[Catflap listening at {}]", at);

    let mut cmd_args = args.values_of("command").unwrap().collect::<Vec<&str>>();

    let cmd = cmd_args.remove(0);

    let err = Command::new(cmd)
        .args(cmd_args)
        .env(env, format!("{}", fd))
        .exec();

    println!("Error running command: {}", err);
}
