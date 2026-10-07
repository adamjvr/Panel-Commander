# X11 integration

X11 is an application/topology integration layer, **not** the DDC transport.

Planned responsibilities:
- correlate XRandR outputs with stable EDID identities;
- observe active applications/windows for profile rules;
- expose hotkeys/notifications;
- never issue monitor I2C traffic through X11-specific APIs when the common hardware backend is available.
