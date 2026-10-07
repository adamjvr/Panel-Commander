# Panel Commander native Linux driver

The userspace backend is intentionally functional first: it works with stock distro kernels via
DRM/sysfs and `/dev/i2c-*`.

This directory is the **native Rust-for-Linux driver track**. It is kept outside the Cargo
workspace because Linux kernel Rust modules are built by Kbuild against a Rust-enabled kernel,
not by normal Cargo.

Current upstream Rust kernel documentation exposes the `kernel::i2c` subsystem and
`module_i2c_driver!`. Panel Commander will use those APIs only where they improve isolation,
serialization, hotplug handling or permissions over the generic userspace backend.

Important design constraint: external monitors on GPU DDC buses are not always instantiated as
ordinary firmware-described I2C clients. The driver therefore must not pretend the monitor is a
normal DT/ACPI I2C peripheral. The next kernel milestone is to bind safely to the DRM connector /
DDC adapter topology and expose a narrow userspace ABI.

The file `panel_commander.rs` is an intentionally minimal build seed, not a falsely claimed
functional hardware driver.
