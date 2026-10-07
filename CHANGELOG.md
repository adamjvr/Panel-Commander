# Changelog

## 0.1.0-dev — OSD correlation R4b qualification hotfix

- Shipped `snapshot.rs` in the exact rustfmt form required by the qualified Rust 1.97 toolchain.
- Promoted read-only `panelctl capabilities`, `probe`, and `snapshot` hardware checks to explicit evidence gates.
- Added `GATES.txt` with per-step result codes to every evidence archive.
- Evidence generation now leaves exactly one timestamped ZIP in `~/Downloads`; SHA-256 is printed instead of creating a sidecar file.

## 0.1.0-dev — R4a qualification hotfix

- Fixed the R4 snapshot diff Clippy `type_complexity` failure by replacing the tuple fingerprint with a named structure.
- Added `scripts/build_test.sh`, a cumulative regression/diagnostic harness that always emits one timestamped evidence ZIP in `~/Downloads`.
- Added automated Git/toolchain/DRM/I2C/DP-AUX diagnostics and optional read-only hardware probe/capabilities/snapshot capture.
- Established the policy that all future Panel Commander passes must ship and pass the evidence harness.

## 0.1.0-dev — XG27WCMS OSD correlation R4

- Added `panelctl snapshot <connector> <output.json>` for read-only, reproducible hardware-state capture.
- Added `panelctl diff <before.json> <after.json>` for offline VCP state correlation.
- Snapshot JSON records monitor identity, selected transport/method, raw capabilities, MCCS/model identity, all advertised VCP results, enum values, and per-code errors.
- Diff refuses to correlate different monitor identities and ignores non-state metadata such as capture timestamps.
- Added regression tests for snapshot change detection.
- Kept all ASUS-private writes locked while semantic characterization is in progress.

## 0.1.0-dev — XG27WCMS live characterization R3

- Promoted the physical XG27WCMS capability string to hardware evidence.
- Added parsing of per-VCP advertised discrete values, including 16-bit values such as `FC(004F)` and `FD(0007)`.
- Changed normal probe/scan behavior to prefer the monitor-advertised VCP set instead of only the forensic seed list.
- Added explicit non-ASUS vs unsupported-ASUS model classification.
- Locked every ASUS-private write until value semantics are hardware-verified.
- Updated the XG27WCMS model database with MCCS 2.2, 42 advertised VCPs/value sets, live read state, and verified DP-AUX routing.

## 0.1.0-dev — Linux DP-AUX routing R2

- Verified XG27WCMS DDC/CI on the connector-associated AMDGPU DP-AUX adapter (`AUS 275E`, brightness 65/100).
- Added ordered per-connector DDC transport candidates instead of assuming the DRM `ddc` symlink is always usable for DDC/CI.
- Added DRM connector ↔ `drm_dp_auxN` ↔ AUX-backed `/dev/i2c-*` correlation.
- Added read-only candidate liveness validation and automatic fallback.
- Added `I2C_RDWR` transport with file-I/O fallback and contextual Linux DDC errors.
- Updated `panelctl` and `panel-commanderd` to use validated connector transports.

## 0.1.0-dev — bootstrap

- Created the Panel Commander Rust workspace.
- Implemented DDC/CI Get VCP / Set VCP / capabilities packet encoding.
- Implemented VCP reply checksum validation.
- Implemented EDID identity parsing.
- Added ASUS XG27WCMS (`AUS275E`) model knowledge from the forensic baseline.
- Added Linux DRM/sysfs connector discovery and DDC bus mapping.
- Added Linux `/dev/i2c-*` transport.
- Added `panelctl` list/probe/get/set/scan/capabilities commands.
- Guarded writes behind explicit `--write`; blocked blind E4/E5 writes.
- Added read-only daemon IPC commands.
- Added macOS native-transport boundary.
- Added Rust-for-Linux native-driver seed and integration architecture.
