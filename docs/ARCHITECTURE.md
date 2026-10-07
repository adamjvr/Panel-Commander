# Panel Commander architecture

```text
GUI / COSMIC applet / panelctl
            |
      profile engine
            |
     monitor model layer
      /             \
  MCCS core      vendor modules
                    ASUS
                     |
               XG27WCMS model
            |
        DDC transport
       /             \
 Linux DRM+i2c     macOS IOKit
      |
 optional Rust-for-Linux native driver
```

## Rules

1. DDC transport is independent of X11/Wayland/COSMIC.
2. Persistent monitor identity is EDID-centered.
3. DDC traffic is serialized per physical transport.
4. Standard MCCS and vendor-private semantics are distinct types.
5. ASUS private E4/E5 are never assigned a universal meaning.
6. GUI is unprivileged.
7. Read-only probing is the default characterization mode.
8. Hardware writes require an explicit user action.
9. Optional HID/accessory support remains separate from monitor DDC.
10. macOS and Linux share protocol/model logic; only native transport differs.
