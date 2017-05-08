extern crate hyper;

use std::os::unix::io::FromRawFd;
use std::process::exit;
use hyper::net::HttpListener;
use hyper::server::{Server, Request, Response};

fn main() {
    std::env::var("LISTEN_FD").ok()
        .and_then(|fd| fd.parse().ok())
        .and_then(|fd| {
            Some(Server::new(unsafe {
                HttpListener::from_raw_fd(fd)
            }))
        })
        .unwrap_or_else(|| {
            // At this point, you should attempt to bind to an IP:Port as
            // normal, so the server can still start if not used with
            // Catflap. For this example, we omit that and simply quit.
            println!("Could not bind to FD!");
            exit(1);
        })
        .handle(|_: Request, _: Response| {}) // Answer 200 OK to every request
        .unwrap_or_else(|e| {
            println!("Could not start server! {}", e);
            exit(1);
        });
}
