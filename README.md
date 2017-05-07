# Cat Flap

This is a small CLI tool for unix-likes that creates a TCP socket at the
address you tell it to, then passes its FD index to a child process using the
`LISTEN_FD` environment variable. The child (or any descendants) can then bind
the socket.

The idea is for tools that reload servers, for instance [cargo watch]:

```
$ catflap -- cargo watch
[Watching for changes... Ctrl-C to stop]
[Running 'cargo run']
   Compiling sample-server v0.1.0 (file:///home/code/rust/test)
    Finished dev [unoptimized + debuginfo] target(s) in 0.71 secs
     Running `target/debug/sample-server`
Binding to socket FD 12345
Serving requests...

[[ Some file is changed so the server is reloaded ]]

[Running 'cargo run']
   Compiling sample-server v0.1.0 (file:///home/code/rust/test)
    Finished dev [unoptimized + debuginfo] target(s) in 0.84 secs
     Running `target/debug/sample-server`
Binding to socket FD 12345
Serving requests...

[[ etc ]]
```

Servers that bind to _ports_ might encounter EADDRINUSE and similar errors, as
they attempt to listen on the same address but before the OS has freed them.

## Usage

```
$ catflap [--] <command> [args...]
$ catflap -p 8000 [--] <command> [args...]
$ catflap -h 0.0.0.0 -p 4567 [--] <command> [args...]
```

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

## Etc

Licensed under [MIT](https://passcod.mit-license.org).
Made by [Félix Saparelli](https://passcod.name).

The name is both because it's a small door that you install so that you don't
have to constantly open and close and open and close a bigger door for your
furry companion, and as a play on the `netcat` tool. 
