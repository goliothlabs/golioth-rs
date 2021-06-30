# golioth-rs

An experimental project for using Golioth (currently just) on the nRF9160.

The CoAP implementation that this uses is not compliant (it's not even a CoAP implementation, just a CoAP parser).

I'm looking into switching to the [rust-async-coap](https://github.com/google/rust-async-coap) crate.

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
