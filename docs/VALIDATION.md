# Validation performed in the build environment

Date: 2026-09-19

The source tree was structurally validated before packaging.

- All 7 `Cargo.toml` manifests parse successfully as TOML.
- DDC/CI Get Brightness request computes to:
  `51 82 01 10 AC` (outer checksum seed `6E`).
- DDC/CI Set Brightness=50 computes to:
  `51 84 03 10 00 32 9A`.
- A standard Brightness 50/100 reply computes reply checksum `F2` using
  the DDC/CI virtual-host checksum seed `50`.
- The Rust unit tests include these protocol invariants plus EDID and capability-parser tests.
- Private E4/E5 writes are blocked by the CLI.

## Toolchain limitation of this execution environment

This container does not contain `rustc`, `cargo`, or `rustfmt`, and outbound shell networking is
disabled, so I could not truthfully claim an in-container Rust compile. The repository therefore
includes GitHub CI and `scripts/check.sh`; the first command to run on Rosie/macOS is:

```bash
./scripts/check.sh
```

Any compile issue discovered there should be treated as an M0 bootstrap bug, not worked around.
