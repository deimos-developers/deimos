# Deimos

Deimos is an experimental, Rust-based extension platform for **Command & Conquer:
Yuri's Revenge**. It starts as a small DLL that can be loaded by
[`syringe-rs`](../syringe-rs), and is intended to grow into a modular framework
for hooks, feature flags, configuration, logging, and gameplay extensions.

The project is currently a **pre-alpha skeleton**. It proves the loading path
and the version-reporting path and provides a Rust port of the Syringe ABI. The
default build does not patch game code. Two optional, experimental features
are available: a Phobos-style `version-banner` and a `dev-console` with a small
command registry.

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
- a complete Rust port of the Syringe ABI in `src/syringe_api.rs`: the
  `Registers` file, hook and host declarations, handshake info, feature-flag
  exports and compile-time layout assertions;
- `declhook!`, `declhook_again!`, `declhost!` and `define_hook!` macros
  mirroring `Syringe.h`;
- an empty hook declaration in the default build, meaning no game code is
  modified yet;
- a `DllMain` that reports the DLL version through `OutputDebugStringA`;
- a `SyringeHandshake` export that returns the version for `--handshakes` mode;
- an optional `version-banner` feature that draws the version in the game
  screen like Phobos does for testing builds, with a black background. It was
  verified in game with YR 1.001 both with and without Phobos; when
  `Phobos.dll` is loaded the banner moves below Phobos' testing-build warning,
  otherwise it sits at the top edge;
- an optional `dev-console` feature that opens a small text box in the
  top-left corner with `~`; typing `cash` and pressing Enter gives the local
  player 5000 credits. While the console is open, `Game::KeyboardProcess`
  input is neutralized so the game's hotkeys do not fire. This was verified
  in a skirmish on YR 1.001 (credits changed from 10000 to 15000, and Escape
  closed the console without opening the game's options menu);
- a shared `engine` layer for drawing, hook addresses and the local-player
  economy, so features do not duplicate engine bindings;
- a command registry in `console/commands.rs`, designed so new commands can be
  added without touching keyboard, rendering or hook code;
- compatibility with the current `syringe-rs` loader.

The version string is compiled from `CARGO_PKG_VERSION`, so a release built
from version `0.1.0` reports:

```text
Deimos 0.1.0
```

## Requirements

To build Deimos you need:

- a stable Rust toolchain (1.85 or newer) with a 32-bit Windows target:
  `i686-pc-windows-gnu` is the reference target, and
  `i686-pc-windows-msvc` is also supported;
- MinGW-w64 (`i686-w64-mingw32-gcc`) on `PATH` for the GNU target, or the
  MSVC build tools for the MSVC target;
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

The experimental in-game banner is compiled only when its feature is enabled:

```sh
cargo build --release --target i686-pc-windows-gnu --features version-banner
```

The in-game developer console is also feature-gated:

```sh
cargo build --release --target i686-pc-windows-gnu --features dev-console
```

Features can be combined, for example
`--features "version-banner dev-console"`.

Run the unit tests on any host. The default suite covers the Syringe ABI,
version constants and handshake; enabling `dev-console` adds the pure
input-buffer and command-registry tests:

```sh
cargo test
cargo test --features dev-console
cargo test --all-features
```

Current local counts are 11 tests for the default build and 18 tests with
`dev-console` or `--all-features`.

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

## Continuous integration

The GitHub Actions workflow in `.github/workflows/ci.yml` validates both the
host-testable core and the Windows DLL:

- **Linux**: `cargo fmt --all -- --check`, strict clippy
  (`--all-targets --all-features -- -D warnings`), host tests with default and
  all features, and `cargo check --target i686-pc-windows-gnu --all-features`;
- **Windows**: `cargo test --all-features --target i686-pc-windows-msvc` and a
  release build with `version-banner dev-console`;
- **Linux/MinGW**: an `i686-pc-windows-gnu` release build.

Both Windows-target jobs upload the resulting `deimos.dll` as a build artifact. Run
the local equivalents before opening a pull request:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo check --target i686-pc-windows-gnu --all-features
```

## Installation and usage

Place the loader, the DLL, and the target executable in one directory:

```text
syringe.exe
deimos.dll
gamemd.exe
```

Start the game through the loader:

```sh
syringe.exe gamemd.exe
```

With Wine, the equivalent command is:

```sh
wine syringe.exe gamemd.exe
```

The loader writes `syringe.log`. A successful minimal load contains lines
similar to:

```text
Debug: find_dlls: Recognized DLL: "deimos.dll"
Debug: log_debug_string: Deimos 0.1.0
Debug: run: Done with exit code 0 (0).
```

The `--handshakes` loader option is supported. Deimos answers with `S_OK` and
the version string; the loader then continues normally.

## How it works

Deimos uses Cargo's `cdylib` output and a Windows entry point. The main pieces
are:

1. **Hook declaration section.** Syringe-compatible loaders look for a
   `.syhks00` section containing 16-byte hook declarations. Deimos emits one
   empty declaration in the default build. This is enough for the loader to
   recognize the DLL as a hook host while keeping the build free of game
   patches. `declhook!` emits a real declaration: the hook address, the number
   of bytes to relocate, and a preferred-image-base pointer to the proc's PE
   export name.
2. **Debug output.** `DllMain` calls `OutputDebugStringA` with a static version
   string. `syringe-rs` forwards debug strings from the target process to its
   log, which makes the version visible without a console.
3. **Handshake.** `SyringeHandshake` is exported with the calling convention
   expected by Syringe-compatible loaders. It fills in the structure size and
   writes the version when the loader provides a buffer. The optional
   `ESPModification`, `ZFPreservation` and `ReladdrInstructionFixup` globals are
   exported for loaders that support those features.
4. **Version banner (optional).** With `--features version-banner`, Deimos hooks
   `GScreenClass::DrawText` (0x4F4583), measures the version text with
   `Drawing::GetTextDimensions`, reads the composite surface size through the
   `DSurface` vtable (`GetWidth`/`GetHeight`) and draws the text with
   `Fancy_Text_Print_Wide`, the same APIs Phobos uses. `Drawing::RenderWidth`
   and `RenderHeight` are render/desktop globals and are not the composite
   surface size, so the vtable accessors are used instead. The addresses come
   from Phobos/YRpp and were verified against the YR 1.001 reference executable
   used during development; the feature remains off by default because a
   different executable build can move them.
   The banner fills a black background with `DSurface::FillRect` (vtable slot
   5) and draws the red text over it. Its vertical position is chosen at
   runtime: when `Phobos.dll` is loaded it uses `PHOBOS_TEXT_Y = 50` to stay
   below Phobos' warning, otherwise it uses `DEFAULT_TEXT_Y = 5` at the top
   edge.
5. **Developer console (optional).** With `--features dev-console`, Deimos
   hooks the same `GScreenClass::DrawText` site and `Game::KeyboardProcess`
   (0x55DEE0). The `console` module is split by responsibility:
   `input.rs` owns the bounded ASCII buffer and its tests, `commands.rs` owns
   the command registry, `keyboard.rs` polls keys and edge-detects them,
   `draw.rs` renders the box, and `hooks.rs` contains only the hook procs and
   declarations. `cash` is a normal registry entry that calls
   `HouseClass::GiveMoney(5000)` on `HouseClass::CurrentPlayer`.
   While the console is open, the `KeyboardProcess` hook replaces the input
   DWORD with zero, so the game's hotkeys do not fire. A one-call latch
   swallows the key that closes the console, covering both hook orderings
   (the draw hook and `KeyboardProcess` can run in either order). Mouse input
   and engine paths outside `KeyboardProcess` are not suppressed. The console
   only supports unshifted letters and digits; it is a development aid, not a
   general command shell.

The default DLL intentionally has no hook bodies. The loader loads it even
though it contributes zero hooks; that loader behavior is a small extension
over the original C++ Syringe loader.

The optional feature builds declare `gamemd` in `.syexe00`, so loaders that do
not run a handshake only load them when the executable is named `gamemd.exe`.
Pass `--handshakes` to bypass the host-name check. The declared checksum is
zero: the loader currently compares names only, and Deimos does not yet verify
the executable build. This is the main reason to keep the engine-patching
features behind feature flags.

## Extending the console

Commands are deliberately decoupled from the console UI and the engine. The
registry lives in [`src/console/commands.rs`](src/console/commands.rs):

```rust
pub(crate) static COMMANDS: &[Command] = &[
    Command { name: "cash", handler: cash },
    // Command { name: "heal", handler: heal },
];

fn heal(args: &[&str]) -> CommandResult {
    if !args.is_empty() {
        return CommandResult::Invalid;
    }
    // Call a shared engine binding here.
    CommandResult::Handled
}
```

`execute` splits the line on whitespace, matches the command name
case-insensitively, and passes the remaining arguments to the handler. Add
engine calls under `src/engine/` rather than inside the console state or
keyboard code. On non-Windows hosts the `cash` handler is compiled as a no-op,
so the registry and parser tests run everywhere.

## Verification

The `version-banner` build was run under Wine 11 with a 32-bit YR 1.001
installation, Phobos 0.5.0.0 alpha1 and Ares 3.0p1. `syringe-rs` reported
3057 hooks (the game/Phobos/Ares baseline plus the Deimos hook). After entering
a skirmish, the red `Deimos 0.1.0` label appeared with its black background in
the top-right area, below the Phobos testing-build warning. With `Phobos.dll`
renamed away for a second run, the same build placed the banner at the top
edge instead, confirming the dynamic placement. This confirms the hook ABI,
the hook chain, the drawing and fill calls and the composite-surface
accessors; it does not replace testing on the exact game build a release
targets.

The `dev-console` build was tested in the same environment. In a skirmish,
pressing `~` opened the bordered box at the top-left; typing `cash` and
pressing Enter changed the sidebar credits from `10000` to `15000`. Pressing
Escape while the console was open closed it without opening the game's
options menu, confirming that `KeyboardProcess` input is suppressed. The test
used synthetic `keybd_event` input under Wine, so the same code path still
needs a manual keyboard test on real hardware. The same sequence was repeated
after the console was split into modules; the refactored build still opened
the box, executed `cash` (10000→15000) and closed on Escape without opening
the options menu.

After the console was split into `engine/` and `console/`, the host test suite
grew from 11 to 18 tests (`cargo test --features dev-console`): the extra seven
cover the input buffer and command registry on Linux. The Windows
`dev-console` and `version-banner dev-console` builds were checked with
`cargo fmt --check` and `cargo clippy --all-targets --all-features --target
i686-pc-windows-gnu`; the combined build still has exactly one `.syexe00`
host declaration and both draw hooks are chained at `0x4F4583`.

## Project layout

```text
deimos/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs
│   ├── syringe_api.rs
│   ├── version_banner.rs
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── drawing.rs
│   │   ├── hooks.rs
│   │   └── house.rs
│   └── console/
│       ├── mod.rs
│       ├── input.rs
│       ├── commands.rs
│       ├── keyboard.rs
│       ├── draw.rs
│       └── hooks.rs
├── .github/
│   └── workflows/
│       └── ci.yml
├── README.md
└── LICENSE.md
```

`src/lib.rs` contains the version constants, the empty hook declaration, the
handshake, `DllMain` and the single `.syexe00` host declaration shared by both
features. `src/syringe_api.rs` contains the ABI types, the hook macros and
their tests. `src/version_banner.rs` is the optional Phobos-style overlay.
`src/engine/` contains the shared engine bindings; `src/console/` contains the
console state machine and its feature-gated Windows modules. The project has
no external Rust dependencies.

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

## Related project

[`syringe-rs`](../syringe-rs) is the loader that consumes Deimos. It has its
own `syringe_api` module for parsing `.syhks00`/`.syexe00` records; Deimos
produces those records through `declhook!`/`declhost!`. The two repositories
are intentionally separate crates, so ABI changes must be made in both and
covered by both test suites.

## Interoperability

Deimos follows the Syringe hook-declaration and handshake ABI, but it is not a
drop-in replacement for Phobos or for any other extension platform. Running
multiple hook hosts that patch the same engine functions can produce conflicts.
`syringe-rs` chains multiple hooks registered at the same address, so the
banner can coexist with Phobos mechanically, but the two overlays may draw over
each other. If another platform is already loaded, test Deimos in a separate
configuration first.

Only the 32-bit Windows build is supported. The project is developed on Linux
and tested with Wine, but Wine results are not a substitute for testing on
Windows.

## Contributing

Issues and pull requests are welcome. Before submitting code:

- run `cargo fmt --all -- --check`;
- run `cargo clippy --all-targets --all-features -- -D warnings`;
- run `cargo test --all-features`;
- run `cargo check --target i686-pc-windows-gnu --all-features`;
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
