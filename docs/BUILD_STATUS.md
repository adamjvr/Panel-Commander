# Build status

## Implemented in v0.1.0-dev

### Portable Rust
- DDC/CI checksums and packet encoders
- VCP response decoder
- capabilities packet decoder/parser
- EDID manufacturer/product/serial/name parser
- common transport trait and error model

### ASUS
- XG27WCMS identity (`AUS` / `0x275E`)
- physical MCCS 2.2 capability map with 42 advertised VCP codes
- advertised discrete-value parsing and model database capture
- private VCP evidence map
- model gating and non-ASUS/unsupported-ASUS classification
- explicit E4/E5 ambiguity handling
- private writes locked until hardware semantics are verified

### Linux
- DRM connector discovery under `/sys/class/drm`
- EDID parsing per connector
- connector → ordered `/dev/i2c-*` candidate mapping
- DisplayPort connector → `drm_dp_auxN` → AUX-backed I2C correlation
- read-only DDC candidate validation/fallback
- I2C slave selection at address `0x37`
- Linux `I2C_RDWR` transaction path with file-I/O fallback
- Get VCP, Set VCP, capabilities request
- 50 ms DDC reply/inter-message timing baseline
- `panelctl list/probe/get/set/scan/capabilities`
- default probe/scan follows the monitor-advertised VCP set when capabilities are available
- read-only daemon commands (`LIST`, `GET`, `CAPS`)

### Native driver
- Rust-for-Linux module seed and Kbuild boundary
- optional-driver architecture retained

### macOS
- crate/API boundary for Intel IOFramebuffer and Apple-Silicon IOAVService implementations

## Not yet claimed complete

- complete XG27WCMS write-semantics characterization (read transport, MCCS 2.2 capability set, advertised values, and initial live VCP state are verified)
- macOS hardware transport implementation
- functional Rust-for-Linux DRM/DDC driver
- GUI
- X11 app-profile integration
- generic Wayland integration
- COSMIC applet/settings integration
- packaging/signing/notarization

These are intentionally not marked complete merely because stubs exist.
