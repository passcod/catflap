#![forbid(clippy::pedantic)]
#![deny(clippy::needless_pass_by_value)]
#![allow(
    clippy::cognitive_complexity,
    clippy::shadow_unrelated,
    clippy::similar_names
)]

use std::net::SocketAddr;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::str::FromStr;
use url::{Host, Url};

mod args;
mod sock;

use sock::{Address, Protocol};

fn parse_address(s: &str, auto_port: &mut u16) -> Result<Address, String> {
    use std::net::{IpAddr, ToSocketAddrs};

    let url = Url::parse(s).map_err(|e| e.to_string())?;

    let proto = match url.scheme() {
        "" | "tcp" => Protocol::Tcp,
        "udp" => Protocol::Udp,
        p => return Err(format!("Invalid protocol: {}", p)),
    };

    let host = url
        .host()
        .ok_or_else(|| "Missing hostname or IP".to_string())?;

    let port = url.port().unwrap_or_else(|| {
        let port = *auto_port;
        *auto_port += 1;
        port
    });

    let ip = match host {
        Host::Domain(h) => (h, port)
            .to_socket_addrs()
            .map_err(|e| e.to_string())?
            .next()
            .ok_or_else(|| format!("Resolved zero addresses from {}", h))?,
        Host::Ipv4(i) => SocketAddr::new(IpAddr::V4(i), port),
        Host::Ipv6(i) => SocketAddr::new(IpAddr::V6(i), port),
    };

    Ok(Address { proto, ip })
}

#[allow(clippy::needless_pass_by_value)]
pub fn check_address(s: String) -> Result<(), String> {
    parse_address(&s, &mut 0).map(|_| ())
}

fn main() -> Result<(), String> {
    let args = args::parse();
    let port_base = u16::from_str(args.value_of("port base").unwrap()).map_err(|e| e.to_string())?;
    let mut free_port = port_base;

    let mut addrs = Vec::new();
    for a in args.values_of("addresses").unwrap() {
        addrs.push(parse_address(a, &mut free_port)?);
    }

    if addrs.is_empty() {
        use std::net::{IpAddr, Ipv4Addr};
        addrs.push(Address {
            proto: Protocol::Tcp,
            ip: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000),
        });
    }

    let mut fds = Vec::with_capacity(addrs.len());
    for mut addr in addrs {
        let fd = sock::on(addr).map_err(|e| format!("binding {}: {}", addr, e))?;
        let at = sock::at_port(fd).map_err(|e| format!("naming {}: {}", addr, e))?;

        if let Some(at_port) = at {
            addr.ip.set_port(at_port);
        }

        fds.push((fd, addr));
    }

    let env = args.value_of("env").unwrap();
    let nenv = args.value_of("nenv").unwrap();

    println!(
        "[Catflap listening at {}]",
        fds.iter()
            .map(|(fd, addr)| format!("{}#fd={}", addr, fd))
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
                .map(|(fd, _)| fd.to_string())
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
