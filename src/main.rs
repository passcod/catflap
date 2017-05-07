extern crate clap;

use std::net::{SocketAddr, TcpListener};
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::{Command, exit};
use std::str::FromStr;

mod args;

fn main() {
    let args = args::parse();

    let addr = SocketAddr::from_str(&format!(
        "{}:{}",
        args.value_of("host").unwrap(),
        args.value_of("port").unwrap()
    )).unwrap_or_else(|e| {
        println!("Bad address: {}", e);
        exit(1);
    });

    let socket = TcpListener::bind(addr).unwrap_or_else(|e| {
        println!("Could not bind: {}", e);
        exit(1);
    });

    let fd = socket.as_raw_fd();
    let on = socket.local_addr().unwrap();
    println!("FD {} listening on {}", fd, on);

    let mut cmd_args = args.values_of("command")
        .unwrap()
        .collect::<Vec<&str>>();

    let cmd = cmd_args.remove(0);

    let err = Command::new(cmd)
        .args(cmd_args)
        .env("LISTEN_FD", format!("{}", fd))
        .exec();

    println!("Error running command: {}", err);
}
