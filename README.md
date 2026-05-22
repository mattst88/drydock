# drydock

`drydock` is a tool for querying Portage profiles. It answers not just *what*
a variable is set to, but *where* and *how* that value is defined across the
profile inheritance hierarchy.

`drydock` operates on profile `make.defaults` files only and does not interact
with ebuilds or evaluate package dependencies.

## Setup

`drydock` needs to know where your repositories live. Generate a config
file with:

```sh
drydock config --default --src-path /var/db/repos
```

The default path is `/var/db/repos`. If your repos are elsewhere (e.g. a local
checkout of the Gentoo tree), pass `--src-path` to any command instead of using
a config file:

```sh
drydock eval -p gentoo:default/linux/amd64/23.0 CHOST --src-path ~/gentoo
```

Profile keys use the format `<repo-name>:<path/under/profiles>`. The repo name
comes from `repo-name` in the repository's `metadata/layout.conf`, falling back to
the directory name if that key is absent (as is the case for the main Gentoo
repo).

## Commands

### `eval`

Print the fully-evaluated value of a variable for a given profile:

```sh
drydock eval -p gentoo:default/linux/alpha/23.0 CHOST
# alpha-unknown-linux-gnu

drydock eval -p gentoo:default/linux/amd64/23.0 USE
```

### `blame`

Show where a variable's value comes from. For non-incremental variables, shows
the value annotated with the source file and line:

```sh
drydock blame -p gentoo:default/linux/amd64/23.0 CHOST
```

For incremental variables (USE, IUSE, etc.), a token must be specified to trace
through the inheritance hierarchy:

```sh
drydock blame -p gentoo:default/linux/amd64/23.0 USE:multilib
```

Output identifies which `make.defaults` file in the profile hierarchy is
responsible for setting or unsetting each token.

### `parents`

Print the profile inheritance tree:

```sh
drydock parents gentoo:default/linux/amd64/23.0
```

Pass `--graph` for Graphviz DOT output suitable for rendering with `dot -Tsvg`.

## Building & Installing

Requires a stable Rust toolchain — install via <https://rustup.rs/>.

### Install

```sh
cargo install --path .
```

### Build

```sh
cargo build --release
# binary at target/release/drydock
```

### Run without installing

```sh
cargo run --release -- eval -p gentoo:default/linux/amd64/23.0 CHOST
```
