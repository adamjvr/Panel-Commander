# Milestones

## M0 — bootstrap (this archive)

- [x] Rust workspace
- [x] byte-exact DDC/CI Get/Set encoder
- [x] VCP reply parser/checksum validation
- [x] EDID identity parser
- [x] ASUS XG27WCMS model definition
- [x] Linux DRM connector enumeration
- [x] Linux `/dev/i2c-*` transport
- [x] read-only VCP probe/scanner
- [x] explicit `--write` gate
- [x] daemon IPC seed
- [x] macOS transport boundary
- [x] Rust-for-Linux native-driver seed

## M1 — live XG27WCMS characterization

- [ ] validate connector→DDC mapping on target GPUs
- [ ] capture exact capability string
- [ ] scan VCP 00..FF read-only
- [ ] characterize E0..FF ranges
- [ ] resolve E4/E5 by OSD state/mode
- [ ] test DP/HDMI/USB-C paths
- [ ] determine safe retry/delay policy
- [ ] add recorded protocol fixtures and regression tests

## M2 — production transport

- [ ] robust Linux I2C_RDWR path and adapter quirks
- [ ] hotplug monitor registry
- [ ] per-monitor transaction queues
- [ ] udev permission rules
- [ ] state/cache invalidation
- [ ] safe write policy and rollback for reversible settings

## M3 — macOS native transport

- [ ] Intel IOFramebuffer implementation
- [ ] Apple Silicon IOAVService implementation
- [ ] display↔service correlation
- [ ] universal binary packaging
- [ ] login/background service integration

## M4 — Widget Center

- [ ] shared profile/state engine
- [ ] graphical application
- [ ] profile import/export
- [ ] input/preset/game/color/eye-care controls
- [ ] ICC integration

## M5 — desktop-native integration

- [ ] X11 adapter
- [ ] generic Wayland adapters
- [ ] COSMIC applet/settings integration
- [ ] macOS native desktop integration

## M6 — native driver

- [ ] DRM/DDC adapter binding design
- [ ] Rust-for-Linux functional transport
- [ ] narrow userspace ABI
- [ ] hotplug/serialization in kernel path
- [ ] stock-kernel userspace fallback remains supported
