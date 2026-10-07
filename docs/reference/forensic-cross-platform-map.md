# Windows ↔ macOS architecture cross-map

| Function | Windows evidence | macOS evidence | Linux/Rust destination |
|---|---|---|---|
| Monitor identity | INF/EDID, `AsusDisplayMonitorInfo.dll` | `Display`, `DisplayManager`, CoreDisplay/IOKit | `edid`, `monitor-core` |
| DDC/CI transport | DXVA2 physical-monitor APIs/native helper | Intel IOFramebuffer I2C + Apple Silicon IOAVService I2C | `ddc-i2c-linux`, DRM connector mapper |
| Generic VCP read/write | `GetVCPFeatureAndVCPFeatureReply`, `SetVCPFeature*` | `GetVCP`, `SetVCP`, `DDCMethods` | `mccs` |
| Capabilities string | `DDCCIGetCapabilitiesStringLength`, capability request | `GetCapabilitiesString`, `DDCCapabilitiesRequest` | `mccs::capabilities` |
| ASUS private controls | managed/native method layer | `VCPAPI`, C wrappers and feature plugins | `asus-vcp` model-gated tables |
| Profiles/preset modes | WPF business logic and JSON capability tables | `PresetModePlugin.framework` | `widget-center-core::profiles` |
| OLED features | WPF OLED methods | `OLEDSettingsPlugin.framework` | `asus-vcp::oled` |
| GamePlus | WPF controls | `GamePlusPlugin.framework` | `asus-vcp::gameplus` |
| Eye care | WPF controls | `EyeCarePlugin.framework` | `asus-vcp::eye_care` |
| Power management | Windows settings logic | `PowerManagementPlugin.framework` | `widget-center-core::power` |
| Multi-screen | Windows topology APIs | `MultiScreenPlugin.framework` | `compositor` adapters |
| Hotkeys | WPF/global input | `HotkeySettingsPlugin.framework`, media event taps | frontend/compositor integration |
| App-specific switching | foreground-process logic | `AppTweakerPlugin.framework`, app event library | `widget-center-core::app_rules` |
| Optional visual/AI/AirVision | Windows auxiliary binaries/WebView | AirVision + AnalysisOrb plugins/frameworks | optional, keep outside monitor core |
| Color profiles | XG27WCMS ICM in WHQL package | ASUS ICC resources installed to ColorSync | `color-management` |
| Startup | Windows installer/service/tray behavior | launchd LaunchAgent opens app | desktop autostart integration |

## Key conclusion
Both platforms converge on the same architectural seam: monitor discovery → DDC/CI transport → generic VCP operations → ASUS model-aware feature layer → policy/UI. Windows-specific DXVA2/WMI and macOS-specific IOFramebuffer/IOAVService are transport implementations, not part of the monitor protocol itself.
