# First live probe procedure

This phase is intentionally read-only.

```bash
cargo run -p panelctl -- list
cargo run -p panelctl -- probe --all
cargo run -p panelctl -- capabilities card1-DP-1
cargo run -p panelctl -- scan card1-DP-1 --all-codes | tee probe-output/xg27wcms-vcp.txt
```

Run the same read-only scan with the monitor in each state that can affect private commands:

- SDR/HDR
- every GameVisual/preset mode
- each physical input
- VRR on/off
- ELMB/blur-reduction modes
- dynamic dimming/shadow-boost states

Do not use the blind `set` command on E4/E5. `panelctl` explicitly blocks those writes in v0.1.

Record:
- connector name;
- GPU/driver;
- cable/input path;
- EDID bytes/hash;
- capability string;
- VCP scan;
- monitor firmware version if exposed.
