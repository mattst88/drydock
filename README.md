# drydock
`drydock` is a tool for querying Portage profiles and producing useful diagnostic
output. `drydock` aims to demystify Portage configuration for users by providing
**fast** queries of configuration values and detailed explanations of where those
values came from.

`drydock` aims to not only answer *what* a value set to, but *where* and *how*
that is value defined.

## Commands
Run ```drydock --help``` to see a list of all commands.

### `eval`
Print the value of a variable as it would be seen by Portage. Example:
```
drydock eval USE --profile grunt:base
```

### `blame`
Show the value of a variable annotated with details of where the contents of that
variable are set throughout the profile hierarchy. Example:
```
drydock blame BOARD_COMPILER_FLAGS --profile octopus:base
```

### `parents`
Print a graphviz representation or text tree of a profile's inheritance tree. Example:
```
drydock parents --graph samus:base
```

## Options and Settings
Get started with ```drydock --help``` to see a list of commands. If you're a Chrome OS
developer you probably want to start with ```drydock config --default``` to generate a
default configuration file.

By default `drydock` tries to read from a configuration file under `$XDG_CONFIG_HOME`
or `~/.config/drydock`, but the config file path can be specified with the
`--config-file` argument. All `drydock` settings can be specified as command-line
arguments in addition to the configuration file, check ```drydock --help``` or
```drydock <subcmd> --help``` for more details.

## Building & Installing
`drydock` requires a stable Rust toolchain, best obtained via https://rustup.rs/

### Installing
You can install `drydock` via `cargo` by running
```sh
cargo install --path ${DRYDOCK_CHECKOUT_DIR?}
```

### Building
`drydock` can be built by running
```sh
cargo build --release
```
while in the project directory. The output binary can then be found at
`target/release/drydock` and can be moved to the location of your choosing.

### Running without installing
`drydock` can also be compiled and run directly from the project directory via
```sh
cargo run --release -- ${YOUR_ARGS?}
```
