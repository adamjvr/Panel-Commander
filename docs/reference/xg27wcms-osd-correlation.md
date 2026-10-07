# XG27WCMS OSD correlation workflow

Panel Commander R4 adds reproducible, read-only JSON snapshots and an offline diff command for live semantic characterization.

## Capture

```bash
sudo ./target/debug/panelctl snapshot card1-DP-2 before.json
# Change exactly one monitor OSD setting.
sudo ./target/debug/panelctl snapshot card1-DP-2 after.json
./target/debug/panelctl diff before.json after.json
```

A snapshot records monitor identity, selected Linux transport and method, raw capabilities, MCCS/model identity, and one record for every advertised VCP. Each record stores semantic label, advertised discrete values, result code, VCP type, maximum/current value, and any read error.

`diff` refuses to correlate snapshots whose EDID-derived identities differ. It compares VCP state only; timestamp and other capture metadata do not produce false state changes.

## Characterization discipline

Change one OSD setting at a time. Keep private ASUS writes disabled until a control's value semantics have been confirmed by repeated before/after observations. Preserve useful snapshots as evidence fixtures before promoting a mapping to the model database.
