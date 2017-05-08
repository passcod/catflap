# Catflap

[![Crate release version](https://img.shields.io/crates/v/catflap.svg?style=flat-square)](https://crates.io/crates/catflap)
[![Crate license: MIT](https://img.shields.io/crates/l/catflap.svg?style=flat-square)](https://passcod.mit-license.org)
[![Crate download count](https://img.shields.io/crates/d/catflap.svg?style=flat-square)](https://crates.io/crates/catflap#crate-downloads)
[![Build status (Travis)](https://img.shields.io/travis/passcod/catflap.svg?style=flat-square)](https://travis-ci.org/passcod/cargo-watch)

This is a small CLI tool for unix-likes that creates a TCP socket at the
address you tell it to, then passes its FD index to a child process using the
`LISTEN_FD` environment variable. The child (or any descendants) can then bind
the socket.

The idea is for tools that reload servers, for instance [cargo watch]:

[cargo watch]: https://github.com/passcod/cargo-watch

```
$ catflap cargo watch
[Catflap listening at 127.0.0.1:5000]
[Running 'cargo run']
   Compiling sample-server v0.1.0 (file:///home/code/rust/test)
    Finished dev [unoptimized + debuginfo] target(s) in 0.71 secs
     Running `target/debug/sample-server`
Binding to socket FD 3
Serving requests...

[[ Some file is changed so the server is reloaded ]]

[Running 'cargo run']
   Compiling sample-server v0.1.0 (file:///home/code/rust/test)
    Finished dev [unoptimized + debuginfo] target(s) in 0.84 secs
     Running `target/debug/sample-server`
Binding to socket FD 3
Serving requests...

[[ etc ]]
```

Servers that bind to _ports_ might encounter EADDRINUSE and similar errors, as
they attempt to listen on the same address but before the OS has freed them.
Additionally, because the socket is always bound, requests simply wait for the
program to answer them instead of failing when the server is restarting,
leading to a better development experience.

Often, process supervisors implement this functionality, for example [systemd],
[lithos], or the [Flask dev server][werkzeug]. Catflap is a single-purpose tool
that does this and only this, so it can be used without all the configuration
or dependence on a particular framework, and it can thus be plugged into your
development workspace at very little cost.

[lithos]: https://lithos.readthedocs.io/en/latest/tips/tcp-ports.html
[systemd]: http://0pointer.de/blog/projects/socket-activation.html
[werkzeug]: https://github.com/pallets/werkzeug/blob/a2a5f5a4c04c5b1fb33709bc2cdc297cd8fb46a3/werkzeug/serving.py#L649-L660

## Install

The usual way:

```
$ cargo install catflap
```

Or, to upgrade:

```
$ cargo install --force catflap
```

## Usage

```
$ catflap [--] <command> [args...]
$ catflap -p8000 [--] <command> [args...]
$ catflap -h0.0.0.0 -p4567 [--] <command> [args...]
```

### Command specifics

The `<command>` is executed directly, without passing through a shell, so
shellisms cannot be used directly. Additionally, you'll want to use `--` to
separate catflap options from program options:

```
$ catflap 'foo && bar'
# Will error because 'foo && bar' doesn't exist in PATH

$ catflap sh -c 'foo && bar'
# Will error because '-c' is not a catflap option

$ catflap -- sh -c 'foo && bar'
# Will work!
```

### Port zero

If you specify port zero, the system will pick an unused high port at random.
Catflap prints the socket's actual address right before it execs the given
command, so you can find the right port to connect to.

```
$ catflap -p0 cargo watch
[Catflap listening at 127.0.0.1:55917]
```

## Etc

Licensed under [MIT](https://passcod.mit-license.org).
Made by [Félix Saparelli](https://passcod.name).

The name is both because it's a small door that you install so that you don't
have to constantly open and close and open and close a bigger door for your
furry companion, and as a play on the `netcat` tool. 
