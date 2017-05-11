use clap::{App, AppSettings, Arg, ArgMatches};

pub fn parse() -> ArgMatches<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::DontCollapseArgsInUsage)
        .setting(AppSettings::StrictUtf8)
        .version(env!("CARGO_PKG_VERSION"))
        .usage("catflap [options] [--] <command> [args...]")

        .arg(Arg::with_name("env")
             .short("e")
             .long("env")
             .default_value("LISTEN_FD"))

        .arg(Arg::with_name("host")
             .short("h")
             .long("host")
             .default_value("127.0.0.1"))

        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .default_value("5000"))

        .arg(Arg::with_name("command")
             .multiple(true)
             .required(true))

        .after_help("Creates a TCP socket then passes its descriptor to a child process using an environment variable (from --env, `LISTEN_FD` by default). The child (or any descendants) can then bind the socket.\n\nThis is useful for server auto-reloaders, as it avoids EADDRINUSE and request-dropping.")

        .get_matches()
}
