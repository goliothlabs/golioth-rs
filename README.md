# golioth-rs

An experimental project for using Golioth (currently just) on the nRF9160.

This project is currently in a holding pattern until the Embedded Rust ecosystem has caught up to the necessary networking support.

# What's needed before resurrection? 

Before this project can be revived, two things are necessary:

1. Rust Embedded needs better networking support. This means that the `AsyncRead` and `AsyncWrite` traits
should be either accessible on `#![no_std]` in a widely-used crate or that they should be stabilized in `core`.

    1.1. This also means that the `nrfxlib` crate should be accessible with `AsyncRead` and `AsyncWrite`.

2. There should be a Rust DTLS implementation that supports `#![no_std]` and PSK authentication usable through `AsyncRead` and `AsyncWrite`. This could be written in Rust (e.g. `drogue-iot`) or bound (e.g. `mbedtls`).

When these critera are met, the `Golioth` struct in this crate should be modified to be generic on an `AsyncRead + AsyncWrite` type, which should be passed into the `Golioth::new` function. The issue here is that we can't confirm that the tunnel is being sent over DTLS: maybe there's a different design
that's better?

# Setup

## Bootloader

You will need to flash [spm](https://github.com/nrfconnect/sdk-nrf/tree/master/samples/spm) onto the nRF9160 you use.
This will jump to `0x50000` on boot, which is where the rust binary will be located.

## Config

Insert your device's Golioth PSK ID and PSK into the consts in the `src/config.rs` file.

## Dependencies

#### 1. `flip-link`:

```console
$ cargo install flip-link
```

#### 2. `probe-run`:

Install the *git* version of `probe-run`

```console
$ cargo install --git https://github.com/knurling-rs/probe-run --branch main
```

#### 4. Install the correct target

```console
$ rustup target add thumbv8m.main-none-eabi --toolchain nightly 
```

#### 4. Run!

```console
$ cargo +nightly run
```

or

```console
$ cargo +nightly run --release
```

> Nightly is required in order to setup an alloc error handler.


#### (4. Set `rust-analyzer.linkedProjects`)

If you are using [rust-analyzer] with VS Code for IDE-like features you can add following configuration to your `.vscode/settings.json` to make it work transparently across workspaces. Find the details of this option in the [RA docs].

```json
{
    "rust-analyzer.linkedProjects": [
        "Cargo.toml",
        "firmware/Cargo.toml",
    ]
} 
```

[RA docs]: https://rust-analyzer.github.io/manual.html#configuration
[rust-analyzer]: https://rust-analyzer.github.io/

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
