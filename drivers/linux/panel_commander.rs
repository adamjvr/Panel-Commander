// SPDX-License-Identifier: GPL-3.0-or-later
//! Panel Commander — Rust-for-Linux native driver seed.
//!
//! This is intentionally not part of the normal Cargo workspace. Build through Kbuild in a
//! Rust-enabled Linux kernel tree.
//!
//! Hardware DDC is already functional through the userspace /dev/i2c-* backend. This module
//! becomes the optional native-driver path after DRM/DDC adapter binding is implemented.

use kernel::prelude::*;

module! {
    type: PanelCommander,
    name: "panel_commander",
    author: "Panel Commander contributors",
    description: "Native monitor-control transport for Panel Commander",
    license: "GPL",
}

struct PanelCommander;

impl kernel::Module for PanelCommander {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Panel Commander native driver seed loaded\n");
        Ok(Self)
    }
}

impl Drop for PanelCommander {
    fn drop(&mut self) {
        pr_info!("Panel Commander native driver seed unloaded\n");
    }
}
