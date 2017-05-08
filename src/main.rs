extern crate clap;
extern crate libc;
extern crate nix;

use std::net::SocketAddr;
use std::os::unix::process::CommandExt;
use std::process::{Command, exit};
use std::str::FromStr;

mod args;
mod sock;

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

	let fd = sock::on(addr).unwrap_or_else(|e| {
        println!("Could not bind: {}", e);
        exit(1);
    });

	let at = sock::at(fd).unwrap_or_else(|e| {
        println!("Could not get address: {}", e);
        exit(1);
    });

	println!("[Catflap listening at {}]", at);

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
