# Linux DDC transport routing

## Rosie hardware finding — 2026-10-07

`card1-DP-2` is the ASUS XG27WCMS (`AUS` / `0x275E`). The DRM connector's
`ddc` symlink resolves to `/dev/i2c-6`, reported by AMDGPU as:

    AMDGPU DM i2c hw bus 1

That adapter returns `EIO` even for an EDID transaction to address `0x50`.
The same connector owns `drm_dp_aux1`; its AUX-backed I2C adapter is
`/dev/i2c-11`:

    AMDGPU DM aux hw bus 1

On `/dev/i2c-11` a direct read-only probe succeeded:

- EDID address `0x50`: `AUS`, product `0x275E`, `XG27WCMS`
- DDC/CI address `0x37`, Get VCP `0x10`: result `0x00`, current `65`, max `100`

`card1-DP-1` demonstrated the inverse case: its normal DRM DDC bus works,
while the corresponding AUX adapter timed out. Therefore Panel Commander must
not hardcode either transport family globally.

## Policy

For each connector Panel Commander builds an ordered list of candidates:

1. the DRM connector's normal `ddc` I2C adapter;
2. any I2C adapter whose adapter name matches a `drm_dp_auxN` object owned by
   that same DRM connector.

The first candidate that produces a valid read-only DDC/CI Get VCP reply is
selected. This keeps the policy generic and avoids hardcoded bus numbers.

The Linux DDC transport itself supports `I2C_RDWR` with file-I/O fallback.
No writable VCP is used during transport selection.
