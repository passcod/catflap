# Catflap

[![Crate release version](https://img.shields.io/crates/v/catflap.svg?style=flat-square)](https://crates.io/crates/catflap)
[![Crate license: Artistic 2.0](https://img.shields.io/crates/l/catflap.svg?style=flat-square)](./LICENSE)
[![Crate download count](https://img.shields.io/crates/d/catflap.svg?style=flat-square)](https://crates.io/crates/catflap#crate-downloads)
[![Build status (Travis)](https://img.shields.io/travis/passcod/catflap.svg?style=flat-square)](https://travis-ci.org/passcod/cargo-watch)
![MSRV: none aka latest stable](https://flat.badgen.net/badge/MSRV/latest%20stable/purple)
![MSRV policy: bump is non-breaking](https://flat.badgen.net/badge/MSRV%20policy/non-breaking/orange)

This is a small CLI tool for unix-likes that creates a network socket for the
address you tell it to, then passes its FD index to a child process using an
environment variable. The child (or any descendants) can then bind the socket.

The idea is for tools that reload servers, for instance [cargo watch]:

[cargo watch]: https://github.com/passcod/cargo-watch

```
$ catflap cargo watch
[Catflap listening at 127.0.0.1:5000 (3)]
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

From version 2.0.0, Catflap provides the same environment variable behaviour as
systemd, so you can use it to test services directly.

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
$ catflap [options] -- <command> [args...]

$ catflap --env CUSTOM_VAR -- <command> [args...]
$ catflap --nenv CUSTOM_VAR_IN_SYSTEMD_STYLE -- <command> [args...]

$ catflap --tcp 0.0.0.0 -- <command> [args...]
$ catflap --tcp 10.10.10.10:80 -- <command> [args...]
$ catflap --tcp :443 -- <command> [args...]

$ catflap --udp :443 -- <command> [args...]
$ catflap --udp auto -- <command> [args...]

$ catflap --tcp :80 --tcp :443 --tcp auto --udp :27192 -- <command> [args...]
$ catflap --tcp :80,:443,: --udp :27192 -- <command> [args...]
```

|Option|Default|Description|
|---|---|---|
|`-e`, `--env`|`LISTEN_FD`|Set this variable to a comma-separated list of FDs. `-` to disable.|
|`-E`, `--nenv`|`LISTEN_FDS`|Set this variable to the number of opened sockets. `-` to disable.|
|`-r`, `--raw`| - |Open a raw IPv4 socket for this address and/or port (requires root or `CAP_NET_RAW`).|
|`-t`, `--tcp`| - |Open a TCP/IPv4 socket for this address and/or port.|
|`-u`, `--udp`| - |Open a UDP/IPv4 socket for this address and/or port.|
|`--raw6`, `--tcp6`, `--udp6`| - |Same as the above Raw IP / TCP / UDP, but over IPv6.|

### Breaking change

If the above options do not work, check your version! Catflap recently changed
its interface (in version 2.0.0). Upgrade to get these new goodies, or [consult
the old README](https://github.com/passcod/catflap/tree/v1.1.0).

### Socket specifics

Each socket option can be passed multiple times. For technically reasons, it's
not (yet?) possible to get an interleaving of socket types: all raw sockets
come first, IPv4 then IPv6, then all TCP, then all UDP. The order is
lexicographic, and thus comes in the same ordering as the options print in the
help message.

Opening "low" ports (below 1024) often requires root, or `CAP_NET_BIND_SERVICE`.

Ports not explicitly specified will start at 5000 for each socket type and
increase by 1 for each unspecified port.

To use the default (127.0.0.1 or [::1] with auto-specified port), provide
either a single `:` or `auto`.

To recap:

```
$ catflap --tcp6 : -t : -u : -r : -t : -u : -r : -t :2000 -u :3000 -r :4000 <command>
```

will provide 5000/raw, 5001/raw, 4000/raw, 5000/tcp, 5001/tcp, 2000/tcp,
5000/tcp6, 5000/udp, 5001/udp, 3000/udp as FDs 3, 4, 5, 6, 7, 8, 9, 10, 11, 12
respectively. It has the same behaviour as these next ones (perhaps clearer):

```
$ catflap -t :,:,:2000 --tcp6 : -u :,:,:3000 -r :,:,:4000 <command>
$ catflap -t auto,auto,:2000 --tcp6 auto -u auto,auto,:3000 -r auto,auto,:4000 <command>
$ catflap \
  --tcp auto  \
  --tcp auto  \
  --tcp :2000 \
  --tcp6 auto \
  --udp auto  \
  --udp auto  \
  --udp :3000 \
  --raw auto  \
  --raw auto  \
  --raw :4000 \
  -- <command>
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
Catflap prints each socket's actual address and corresponding FD right before
it execs the given command, so you can find the right port to connect to.

```
$ catflap -t :0 -- cargo watch
[Catflap listening at 127.0.0.1:55917 (3)]
```

## Example servers

These can be built and run directly in the respective folder.
Then simply: `$ curl -i http://localhost:5000`.

- [Hyper only](./integrations/hyper).
- [Using Iron](./integrations/iron).
- [Express on Node.js](./integrations/express).

## Etc

Licensed under [Artistic 2.0](./LICENSE).
Made by [Félix Saparelli](https://passcod.name).

The name is both because it's a small door that you install so that you don't
have to constantly open and close and open and close a bigger door for your
furry companion, and as a play on the `netcat` tool. 
