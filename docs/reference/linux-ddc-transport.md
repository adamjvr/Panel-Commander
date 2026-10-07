# Linux DDC transaction methods

Panel Commander supports two userspace i2c-dev transaction mechanisms:

- `I2C_RDWR` ioctl, preferred in automatic mode;
- normal read/write file I/O after `I2C_SLAVE` selection, used as fallback.

Set `PANEL_COMMANDER_I2C_MODE=rdwr`, `file`, or `auto` for diagnostics.
Automatic mode remembers whichever method succeeds for subsequent operations
on the selected adapter.

Errors include the adapter path, operation, and transfer method so GPU/DDC
routing failures do not collapse into an unqualified `EIO`.
