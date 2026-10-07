# Linux/Rust reconstruction target

## Layering

```text
apps/
  asus-displayctl       CLI/probe/debug tool
  widget-center         desktop GUI
  cosmic-applet         COSMIC integration

crates/
  monitor-core          stable monitor identity + lifecycle model
  edid                  parser + ASUS identity matching
  drm-discovery         /sys/class/drm connector enumeration and EDID↔I2C mapping
  ddc-transport         transport trait, queueing, retry/timing policy
  ddc-i2c-linux         /dev/i2c-* implementation
  mccs                  DDC/CI framing, checksum, VCP, capabilities parser
  asus-vcp              ASUS private/model-gated semantics
  hid-transport         optional USB HID accessory support
  color-management      ICC/profile installation/selection
  widget-center-core    profiles, app switching, persistence, rules
  compositor            compositor-neutral monitor/window integration traits
  x11-backend           XRandR/window/app integration only
  wayland-backend       Wayland protocol adapters only
  cosmic-backend        COSMIC-native settings/window/app integration
```

## Hard architecture rules recovered from ASUS implementations
1. DDC hardware access must not depend on X11 or Wayland.
2. Monitor identity must be stable across compositor sessions: use EDID vendor/product/serial plus connector topology; do not use transient GUI display IDs as persistent identity.
3. Serialize DDC transactions per physical transport/monitor. The mac implementation contains explicit I2C queues and a global DDC queue.
4. Preserve post-transaction delay/retry behavior. Intel macOS code visibly inserts a 20 ms delay in one request path and has DDC-specific delay selection.
5. Private ASUS VCP bytes are model/capability gated. `0xE4` and `0xE5` demonstrably have aliases in the shipped wrapper layer.
6. Separate monitor DDC from optional HID/lightbar/accessory transports.
7. Keep optional AirVision/AI/web/user-account functions outside the monitor-control core.
8. GUI should remain unprivileged. Linux hardware permissions should be solved with udev/group rules or a minimal narrow helper only if unavoidable.

## Linux transport
Use DRM/sysfs for connector discovery and EDID. Map DRM connectors to I2C/DDC buses, then issue DDC/CI over `/dev/i2c-*`. The compositor adapters are for user-visible topology, window tracking, app rules, notifications, and integration—not VCP transport.

## X11
Use XRandR for topology/correlation as needed and EWMH/X11 app/window observation for app-specific profile switching. Hardware writes still go through the common DDC layer.

## Wayland
Avoid XWayland dependency. Use compositor-supported output/window protocols where available and gracefully degrade app-specific switching if a compositor does not expose window identity.

## COSMIC
Build a dedicated adapter against COSMIC’s native protocols/settings surfaces. COSMIC should be an enhanced frontend/integration target, not a fork of the hardware core.
