ToyQL Database Example
======================

This repository contains an ongoing project to build a simple "toy" database
to share some learned wisdom of systems engineering with the broader world.

The blog explaining this project and its progress is at:

[Let's Build Systems](https://letsbuild.systems)

This project is written primarily in `rust`.  To play with this repository
locally, you must first have `cargo` installed. If you don't already have it,
you can do so via:

```sh
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustc -V
```

You can then verify your configuration via:

```sh
./validate
```

Once you've done that, you can run the database via:

```sh
cargo run
```

This project is not intended to receive PRs at this stage (its flaws are
mostly deliberate and intrinsic to the progress of the blog).
