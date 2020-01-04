use crate::{check_socket_v4, check_socket_v6};
use clap::{App, AppSettings, Arg, ArgMatches};

pub fn parse() -> ArgMatches<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::DontCollapseArgsInUsage)
        .setting(AppSettings::StrictUtf8)
        .version(env!("CARGO_PKG_VERSION"))
        .usage("catflap [options] -- <command> [args...]")
        .arg(Arg::with_name("env")
             .short("e")
             .long("env")
             .help("Fills this variable with a comma-separated list of FDs. - to disable.")
             .default_value("LISTEN_FD")
        )
        .arg(Arg::with_name("nenv")
             .short("E")
             .long("nenv")
             .help("Fills this variable with the number of opened sockets. - to disable.")
             .default_value("LISTEN_FDS")
        )
        .arg(Arg::with_name("raw")
            .short("r")
            .long("raw")
            .value_name("[ip]:[port]")
            .multiple(true)
            .use_delimiter(true)
            .help("Opens an IPv4 raw socket for this address and/or port.")
            .validator(check_socket_v4)
        )
        .arg(Arg::with_name("raw6")
            .long("raw6")
            .value_name("[ip]:[port]")
            .multiple(true)
            .use_delimiter(true)
            .help("Opens an IPv6 raw socket for this address and/or port.")
            .validator(check_socket_v6)
        )
        .arg(Arg::with_name("tcp")
            .short("t")
            .long("tcp")
            .value_name("[ip]:[port]")
            .multiple(true)
            .use_delimiter(true)
            .help("Opens an IPv4 TCP socket for this address and/or port.")
            .validator(check_socket_v4)
        )
        .arg(Arg::with_name("tcp6")
            .long("tcp6")
            .value_name("[ip]:[port]")
            .multiple(true)
            .use_delimiter(true)
            .help("Opens an IPv6 TCP socket for this address and/or port.")
            .validator(check_socket_v6)
        )
        .arg(Arg::with_name("udp")
            .short("u")
            .long("udp")
            .value_name("[ip]:[port]")
            .multiple(true)
            .use_delimiter(true)
            .help("Opens an IPv4 UDP socket for this address and/or port.")
            .validator(check_socket_v4)
        )
        .arg(Arg::with_name("udp6")
            .long("udp6")
            .value_name("[ip]:[port]")
            .multiple(true)
            .use_delimiter(true)
            .help("Opens an IPv6 UDP socket for this address and/or port.")
            .validator(check_socket_v6)
        )
        .arg(Arg::with_name("command")
            .multiple(true)
            .required(true)
        )
        .after_help("Creates IP sockets and passes their descriptors to a child process using environment variables (by default, in systemd style, with the number of provided FDs in `LISTEN_FDS` — FDs start at 3). The child (or any descendants) can then bind the socket. This is useful for server auto-reloaders, as it avoids EADDRINUSE and request-dropping.\n\nAuto-filled ports (when not provided) start from 5000. You can write `auto` or `:` to use the default (127.0.0.1 or [::1], as appropriate, with auto-filled ports). Regardless of the order you pass options in, they are grouped lexicographically (raw before tcp before udp, v4 before v6) before being opened.\n\nDefault is to open one TCP v4 socket on port 5000.")

        .get_matches()
}
