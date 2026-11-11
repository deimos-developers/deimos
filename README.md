# Deimos

Deimos is an experimental, Rust-based extension platform for **Command & Conquer:
Yuri's Revenge**. It starts as a small DLL that can be loaded by
[`syringe-rs`](../syringe-rs), and is intended to grow into a modular framework
for hooks, feature flags, configuration, logging, and gameplay extensions.

The project is currently a **pre-alpha skeleton**. It proves the loading path
and the version-reporting path; it does not yet patch game code or implement
gameplay features.

> **Warning**
>
> Deimos is under active development. Back up your game installation and your
> save files before testing it. Use it in private or single-player sessions
> first. Do not use early builds in competitive online play.

> **EA disclaimer**
>
> **EA has not endorsed and does not support this product.** Deimos is an
> unofficial community project. It is not affiliated with, sponsored by, or
> approved by Electronic Arts Inc. or any of its subsidiaries. Command &
> Conquer, Yuri's Revenge, and all related names, marks, and assets are the
> property of their respective owners.

## Current state

The current build provides:

- a 32-bit Windows `cdylib` written in Rust;
- a valid `.syhks00` hook-declaration section, so Syringe-compatible loaders
  recognize the DLL as a hook host;
- an empty hook declaration, meaning no game code is modified yet;
- a `DllMain` that reports the DLL version through `OutputDebugStringA`;
- an optional `SyringeHandshake` export for `--handshakes` mode;
- compatibility with the current `syringe-rs` loader.

The version banner is compiled from `CARGO_PKG_VERSION`, so a release built
from version `0.1.0` reports:

```text
Deimos 0.1.0
```

## Requirements

To build Deimos you need:

- a stable Rust toolchain with the `i686-pc-windows-gnu` target;
- MinGW-w64 (`i686-w64-mingw32-gcc`) on `PATH`;
- Cargo.

To run Deimos you need:

- a 32-bit installation of *Command & Conquer: Yuri's Revenge* (version 1.001
  is the reference target);
- a compatible loader, currently [`syringe-rs`](../syringe-rs);
- the Deimos DLL placed in the directory from which the loader is started.

Deimos does not ship or require any original game assets.

## Building

From the repository root:

```sh
rustup target add i686-pc-windows-gnu
cd deimos
cargo build --release --target i686-pc-windows-gnu
```

The crates.io package is named `deimos-rs`; the library target and the
resulting DLL are still named `deimos`.

The output is:

```text
deimos/target/i686-pc-windows-gnu/release/deimos.dll
```

The build script adds the `--enable-stdcall-fixup` linker option for the
Windows GNU target so the `DllMain` export is usable by the Windows loader.

To change the reported version, update `version` in `Cargo.toml` and rebuild.
No separate version resource is maintained yet.

## Installation and usage

Place the loader, the DLL, and the target executable in one directory:

```text
syringe-rs.exe
deimos.dll
game.exe
```

Start the game through the loader:

```sh
syringe-rs.exe game.exe
```

With Wine, the equivalent command is:

```sh
wine syringe-rs.exe game.exe
```

The loader writes `syringe-rs.log`. A successful minimal load contains lines
similar to:

```text
Debug: find_dlls: Recognized DLL: "deimos.dll"
Debug: log_debug_string: Deimos 0.1.0
Debug: run: Done with exit code 0 (0).
```

The `--handshakes` loader option is supported. Deimos answers with `S_OK` and
a short message; the loader then continues normally.

## How it works

Deimos uses Cargo's `cdylib` output and a Windows entry point. Three pieces are
important:

1. **Hook declaration section.** Syringe-compatible loaders look for a
   `.syhks00` section containing 16-byte hook declarations. Deimos emits one
   empty declaration. This is enough for the loader to recognize the DLL as a
   hook host while keeping the current build free of game patches.
2. **Debug output.** `DllMain` calls `OutputDebugStringA` with a static version
   string. `syringe-rs` forwards debug strings from the target process to its
   log, which makes the banner visible without a console.
3. **Handshake.** `SyringeHandshake` is exported with the calling convention
   expected by Syringe-compatible loaders. It fills in the structure size and
   writes a message when the loader provides a buffer.

The current DLL intentionally has no hook bodies and no feature flags. The
loader loads it even though it contributes zero hooks; that loader behavior is
a small extension over the original C++ Syringe loader.

## Project layout

```text
deimos/
├── Cargo.toml
├── build.rs
├── src/
│   └── lib.rs
├── README.md
└── LICENSE.md
```

`src/lib.rs` contains the hook declaration, handshake, and `DllMain`. The
project has no external Rust dependencies.

## Roadmap

Deimos is intended to become a feature-rich platform rather than a single hook
DLL. Planned areas, in no particular order, include:

- hook registration and safe Rust wrappers around game objects;
- feature flags and runtime configuration;
- structured logging and diagnostics;
- modular extension crates;
- compatibility and interoperability tests with existing extension platforms;
- documentation and examples for extension authors.

These are goals, not commitments. Interfaces will change while the project is
in pre-alpha.

## Interoperability

Deimos follows the Syringe hook-declaration and handshake ABI, but it is not a
drop-in replacement for Phobos or for any other extension platform. Running
multiple hook hosts that patch the same engine functions can produce conflicts.
If another platform is already loaded, test Deimos in a separate configuration
first.

Only the 32-bit Windows build is supported. The project is developed on Linux
and tested with Wine, but Wine results are not a substitute for testing on
Windows.

## Contributing

Issues and pull requests are welcome. Before submitting code:

- run `cargo fmt --check`;
- run `cargo clippy --all-targets`;
- build and test the `i686-pc-windows-gnu` target;
- do not add original game assets, leaked code, or code of unclear provenance;
- discuss ABI or loader-visible changes before implementing them.

By contributing, you agree that your contribution is licensed under the same
terms as the project.

## Credits

Deimos builds on ideas and interfaces from the Command & Conquer modding
community, including the Syringe and SyringeEx loaders and the YRpp project.
Phobos demonstrated how a large, community-maintained Yuri's Revenge extension
platform can be organized; Deimos is an independent implementation and is not
affiliated with Phobos.

The Rust toolchain, MinGW-w64, and Wine are used for development and testing.
All trademarks remain the property of their respective owners.

## Legal and license

Deimos is free software distributed under the **GNU General Public License,
version 3 only** (`GPL-3.0-only`). The full license text is in
[LICENSE.md](LICENSE.md). If you distribute a modified DLL, you must comply
with the GPL, including making the corresponding source available.

Deimos is an unofficial, non-commercial community project. It is intended to be
used in accordance with Electronic Arts' Command & Conquer modding guidelines.
It contains no original game code or assets. Electronic Arts has not endorsed
and does not support this product.

The software is provided **as is**, without warranty of any kind. The authors
are not responsible for damage to game installations, save files, or systems.
