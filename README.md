# Panel Commander

**Open-source native monitor control for Linux and macOS.**

Panel Commander is a clean-room Rust implementation of a cross-platform monitor-control stack.
The first hardware target is the ASUS XG27WCMS (`AUS275E`), reconstructed from observable
DDC/CI behavior and independently analyzed Windows/macOS software.

This repository deliberately does **not** depend on ASUS DisplayWidget Center binaries.

## Current milestone

`v0.1.0-dev` is the first working vertical slice:

- portable DDC/CI packet encoder/decoder;
- EDID parser and stable monitor identity;
- ASUS/XG27WCMS model database;
- Linux DRM/sysfs monitor discovery;
- native Linux `/dev/i2c-*` DDC transport;
- read-only hardware probe and VCP scanner;
- explicitly gated VCP writes through `panelctl ... set ... --write`;
- daemon protocol skeleton;
- macOS transport boundary ready for the recovered Intel/Apple-Silicon implementations;
- Rust-for-Linux native-driver skeleton and architecture notes;
- X11, Wayland and COSMIC integration plan kept above the hardware transport.

## Safety model

The default probe path is read-only. `panelctl set` refuses to write unless `--write` is
present. Private ASUS VCP bytes are model-gated and are **not** treated as globally meaningful.

## Build

```bash
cargo build --workspace
cargo test --workspace
```

On Linux, access to `/dev/i2c-*` is required. During development this may require appropriate
udev/group permissions.

## First hardware pass

```bash
panelctl list
panelctl probe --all
panelctl get card1-DP-1 0x10
panelctl scan card1-DP-1
```

To deliberately write:

```bash
panelctl set card1-DP-1 0x10 50 --write
```

## Repository layout

```text
crates/panel-commander-core/   protocol, EDID, types
crates/panel-commander-asus/   ASUS model/private-VCP knowledge
crates/panel-commander-linux/  DRM/sysfs + /dev/i2c-* transport
crates/panel-commander-macos/  macOS transport boundary
apps/panelctl/                 CLI/probe tool
apps/panel-commanderd/         service/IPC skeleton
drivers/linux/                 Rust-for-Linux native driver work
integrations/                  X11/Wayland/COSMIC/macOS plans
docs/                          forensic provenance and architecture
```

## Clean-room boundary

The implementation is based on protocol behavior, public platform interfaces, EDID data,
symbol/API observations, and independently reconstructed constants. Decompiled vendor source is
not imported into this tree.

See `docs/PROVENANCE.md`.
