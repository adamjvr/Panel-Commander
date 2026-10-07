# Validation

Panel Commander passes are qualified on the target workstation with:

```bash
sudo -v
./scripts/build_test.sh
```

The harness is intentionally non-mutating. It checks formatting rather than rewriting source and records each gate in `GATES.txt`.

Required regression gates:

- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo build --workspace`
- `panelctl list`
- read-only target-monitor `probe`
- read-only target-monitor `capabilities`
- read-only target-monitor JSON `snapshot`

Every run packages one timestamped `Panel-Commander-BuildTest-*.zip` in `~/Downloads`, on success or failure. The archive contains `RESULT.txt`, `GATES.txt`, individual logs, Git/toolchain provenance, DRM/I2C/DP-AUX topology, capabilities evidence, and the JSON hardware snapshot. The ZIP SHA-256 and final result code are printed to the terminal; no loose diagnostic logs or checksum sidecar are created.
