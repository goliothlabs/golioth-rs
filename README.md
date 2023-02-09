# golioth-rs

An experimental project for using Golioth on the nRF9160 cellular SiP.  This repo is built on the [Embassy]
async framework and [nrf-modem].  This repo currently requires the nightly compiler.

# Setup

## Bootloader

You will need to flash a Secure Partition Manager (SPM) onto the nRF9160.  For quickstart, an SPM is included in this repo. Embassy can run in secure 
mode on the nRF9160; however, the modem library must operate in non-secure (NS) mode which means we need the SPM to transition into NS mode
on boot. This will jump to `0x50000` on boot, which is where the rust binary will be located.

## Config

Insert your device's Golioth PSK ID and PSK in the [src/config.rs] file.

## Dependencies

#### 1. `Clang` and `GCC ARM Toolchain`:
```console
$ sudo apt install llvm-dev libclang-dev clang
$ sudo apt install gcc-arm-none-eabi
```

#### 2. `flip-link`:

```console
$ cargo install flip-link
```

#### 3. `probe-run`:

Install [probe-run] which is used for the project runner in [.cargo/config.toml]

```console
$ cargo install probe-run
```

#### 4. Set compiler default to `nightly
`
```console
$ rustup override set nightly
```

#### 5. Install the correct target

```console
$ rustup target add thumbv8m.main-none-eabi 
```

#### 6. Run!

```console
$ cargo run --bin sensor_stream
```

or

```console
$ cargo run --release --bin sensor_stream
```

> Nightly is required in order to set up an alloc error handler and for Embassy.


#### (7. OPTIONAL Set `rust-analyzer.linkedProjects`)

If you are using [rust-analyzer] with VS Code for IDE-like features you can add following configuration to your `.vscode/settings.json` to make it work transparently across workspaces. Find the details of this option in the [RA docs].

```json
{
    "rust-analyzer.linkedProjects": [
        "Cargo.toml",
        "firmware/Cargo.toml"
    ]
} 
```
[Embassy]: https://github.com/embassy-rs/embassy
[nrf-modem]: https://docs.rs/nrf-modem/0.2.0/nrf_modem/
[probe-run]: https://crates.io/crates/probe-run
[RA docs]: https://rust-analyzer.github.io/manual.html#configuration
[rust-analyzer]: https://rust-analyzer.github.io/

[src/config.rs]: src/config.rs
[.cargo/config.toml]: .cargo/config.toml

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
