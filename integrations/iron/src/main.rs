extern crate hyper;
extern crate iron;

use hyper::net::HttpListener;
use iron::prelude::*;
use iron::Protocol;
use iron::status;
use std::os::unix::io::FromRawFd;
use std::process::exit;

fn main() {
    fn hello_world(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    let listener = std::env::var("LISTEN_FD").ok()
        .and_then(|fd| fd.parse().ok())
        .and_then(|fd| Some(unsafe {
            HttpListener::from_raw_fd(fd)
        }))
        .unwrap_or_else(|| {
            // At this point, you should attempt to bind to an IP:Port as
            // normal, so the server can still start if not used with
            // Catflap. For this example, we omit that and simply quit.
            println!("Could not bind to FD!");
            exit(1);
        });

    let _server = Iron::new(hello_world)
        .listen(listener, Protocol::http())
        .unwrap();

    println!("Listening");
}
