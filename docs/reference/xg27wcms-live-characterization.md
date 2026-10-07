# XG27WCMS live characterization — 2026-10-07

Panel Commander successfully identified and communicated with a physical ASUS XG27WCMS over native DisplayPort on AMDGPU.

## Identity

- EDID manufacturer: `AUS`
- EDID product: `0x275E`
- EDID monitor name: `XG27WCMS`
- MCCS version: `2.2`
- capability model: `XG27WCMS`

## Linux transport result

The DRM connector was `card1-DP-2`.

The DRM `ddc` symlink led to `/dev/i2c-6` (`AMDGPU DM i2c hw bus 1`), but direct EDID and DDC/CI transactions on that adapter returned `EIO`.

The same connector owns `drm_dp_aux1`, whose AUX-backed I2C adapter is `/dev/i2c-11` (`AMDGPU DM aux hw bus 1`). On that path:

- EDID address `0x50` returned a valid `AUS 275E` EDID.
- DDC/CI address `0x37` returned a valid Get VCP `0x10` response.
- brightness was `65/100`.
- Panel Commander R2b automatically selected `/dev/i2c-11` using `I2C_RDWR`.

This is why Linux discovery validates per-connector candidates instead of blindly trusting the DRM `ddc` symlink or globally preferring DP-AUX.

## Advertised capability string

```text
(prot(monitor) type(LCD)model(XG27WCMS) cmds(01 02 03 07 0C F3) vcp(02 04 05 08 10 12 14(05 06 08 0B) 16 18 1A 52 60(11 1A 0F) 62 72(50 64 78 8C A0) 86(01 02 0B 0D 0F) 8A 8D(01 02) AC AE B6 C6 C8 CC(01 02 03 04 05 06 07 08 09 0A 0C 0D 11 12 14 1A 1E 1F 23 24 30 31) D6(01 05) DF DC(01 02 03 04 05 06 07 08 09 0A) DD(00 01) E0 E1(00 01) E2(00 01 02 03) E3(00 02 07 09 0A 0B 0C) E4(00 01 02 03 04 05) E5(00 01 02 03 04) E6 E7(00 01) E8(01 02 03 04 05 06 07 08) EA(00 01 02) EB(00 01 02 03 04 05 06 07) EC(01 FF) EE(00 01 02 03 04 05) FC(004F) FD(0007))mccs_ver(2.2)asset_eep(32)mpu(01)mswhql(1))
```

The monitor advertises 42 VCP feature codes. R3 parses both the feature codes and their discrete advertised value lists.

## Verified live read state

| VCP | Current / max | Result |
| --- | --- | --- |
| 0x10 brightness | 65 / 100 | supported |
| 0x12 contrast | 80 / 100 | supported |
| 0x14 color preset | 5 / 11 | supported |
| 0x16 red gain | 100 / 100 | supported |
| 0x18 green gain | 100 / 100 | supported |
| 0x1A blue gain | 100 / 100 | supported |
| 0x60 input source | 15 / 20 | supported |
| 0x62 audio volume | 50 / 100 | supported |
| 0x8A saturation | 50 / 100 | supported |
| 0x90 hue | — | unsupported result 0x01 |
| 0xD6 power mode | 1 / 4 | supported |
| 0xDF MCCS version | 514 / 65535 | supported |
| 0xE0 ASUS overdrive | 10 / 20 | supported |
| 0xE1 ASUS power saving | 0 / 1 | supported |
| 0xE2 ASUS screen saver | 2 / 3 | supported |
| 0xE3 ASUS crosshair | 0 / 11 | supported |
| 0xE4 ASUS private/multiplexed | 0 / 5 | supported |
| 0xE5 ASUS private/multiplexed | 0 / 255 | supported |
| 0xE6 ASUS blue-light filter | 0 / 4 | supported |
| 0xE7 ASUS display alignment | 0 / 1 | supported |
| 0xE8 ASUS GamePlus position | — | unsupported result 0x01 |
| 0xEA ASUS FPS counter | 0 / 2 | supported |
| 0xEC ASUS reset mode default | — | unsupported result 0x01 |
| 0xED ASUS proximity sensor | — | unsupported result 0x01 |
| 0xF9 ASUS screen saver alias | — | unsupported result 0x01 |

## Write policy

No ASUS-private VCP is write-enabled yet. The monitor advertising a code or value list does not by itself establish the user-visible meaning of every value. E4/E5 in particular are multiplexed in the ASUS binaries. Panel Commander requires hardware correlation against OSD state before enabling private writes.
