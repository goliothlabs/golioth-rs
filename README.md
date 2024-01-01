# golioth-rs

An experimental project for using Golioth on the nRF9160 cellular SiP.  This repo is built on the [Embassy]
async framework and [nrf-modem].  This repo currently requires Rust 1.75 or higher.

# Setup

## Bootloader

Most nRF9160 devices come loaded with the MCUBoot bootloader.  Using this repo will overwrite MCUBoot.  MCUBoot will need to be replaced before going back to the Nordic SDK.  The easiest way to restore it 
is to flash a binary that includes the MCUBoot in it.  MCUBoot is not needed for this repo.

You will need to flash a Secure Partition Manager (SPM) onto the nRF9160.  For quickstart, an SPM is included in this repo. Embassy can run in secure 
mode on the nRF9160; however, the modem library must operate in non-secure (NS) mode which means we need the SPM to transition into NS mode.  This will jump to `0x50000` on boot, which is where the rust binary will be located.

## Config

Insert your device's Golioth PSK ID and PSK in the [src/config.rs] file.

## Dependencies

#### 1. `Clang` and `GCC ARM Toolchain`:
Nordic's nrfxlib is written in C, we need Clang and the ARM GCC toolchain for generating bindings

Unix
```console
$ sudo apt install llvm-dev libclang-dev clang
$ sudo apt install gcc-arm-none-eabi
```

Windows

Download and install [GCC toolchain] and [LLVM-Clang], then make sure they are in the `Path` environment variable.  You can test this by running these commands.
```console
$ arm-none-eabi-gcc --version
$ clang --version
```
#### 2. `flip-link`:
This flips the stack for overflow protection
```console
$ cargo install flip-link
```

#### 3. `probe-run`:

Install [probe-run] which is used for the project runner in [.cargo/config.toml]

```console
$ cargo install probe-run
```

#### 4. Install [llvm-tools] 

```console
$ rustup component add llvm-tools-preview
```

#### 5. Use the current official nightly compiler for Embassy
Nightly is required in order to set up an alloc error handler and for Embassy.  This is handled in `rust-toolchain.toml`

#### 6. Install the correct target

```console
$ rustup target add thumbv8m.main-none-eabi 
```

#### 7. Run!
Build profiles, such as `release`, can be configured in [Cargo.toml]
```console
$ cargo run --bin sensor_stream
```

or

```console
$ cargo run --release --bin sensor_stream
```
#

#### 8. OPTIONAL Set `rust-analyzer.linkedProjects`

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
[GCC Toolchain]: https://developer.arm.com/downloads/-/gnu-rm
[LLVM-Clang]: https://github.com/llvm/llvm-project/releases/tag/llvmorg-16.0.0
[llvm-tools-preview]: https
[probe-run]: https://crates.io/crates/probe-run
[RA docs]: https://rust-analyzer.github.io/manual.html#configuration
[rust-analyzer]: https://rust-analyzer.github.io/

[src/config.rs]: src/config.rs
[.cargo/config.toml]: .cargo/config.toml
[Cargo.toml]: Cargo.toml

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
