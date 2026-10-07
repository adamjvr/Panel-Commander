# Clean-room provenance

Panel Commander is a from-scratch implementation.

The initial XG27WCMS model knowledge was derived from:

- the monitor's published/observable EDID identity (`AUS275E`);
- protocol behavior and DDC/CI/MCCS packet formats;
- static observation of independently shipped Windows and macOS applications;
- exported symbol/type names and reconstructed immediate VCP constants;
- public operating-system APIs and kernel interfaces;
- live hardware responses obtained by Panel Commander's own probe tooling.

No ASUS source file is copied into this repository.

## Reconstructed private VCP evidence

Static analysis identified these ASUS wrapper mappings:

| Code | Observed semantic |
|---|---|
| E0 | Overdrive |
| E1 | Power saving |
| E2 | Screen-saver variant |
| E3 | Crosshair |
| E4 | multiplexed: GamePlus timer / InputRange alias |
| E5 | multiplexed: Dynamic dimming / ShadowBoost alias |
| E6 | Blue-light filter |
| E7 | Display alignment |
| E8 | GamePlus position |
| EA | FPS counter |
| EC | reset mode default |
| ED | proximity sensor |
| F9 | screen-saver variant |

E4 and E5 remain intentionally unresolved until live XG27WCMS characterization determines the
model/mode-specific encoding.
