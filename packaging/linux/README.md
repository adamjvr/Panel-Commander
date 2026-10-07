# Linux packaging

Panel Commander needs read/write access to the DDC I2C adapter exposed for each DRM connector.

Do **not** ship a blanket world-writable `/dev/i2c-*` rule.

Recommended distro integration:
1. create/use an `i2c` group or a narrowly scoped udev rule;
2. grant the logged-in desktop user access to the connector's DDC adapter;
3. run `panel-commanderd` as the user, not as root;
4. keep the stock `/dev/i2c-*` backend available even when the optional native kernel module is installed.

The included systemd user unit is a development seed; package paths should be adjusted by each
distribution package.
