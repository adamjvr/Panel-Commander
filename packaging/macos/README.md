# macOS packaging

Target layout:

```text
Panel Commander.app
  Contents/MacOS/panel-commander
  Contents/Resources/...
  helper/service as required by final transport design
```

The monitor-control path itself should not require a kernel extension. The recovered ASUS package
used user-space IOKit/DDC access and a LaunchAgent. A DriverKit/System Extension target remains
appropriate only for device classes Apple actually exposes through DriverKit (for example optional
USB/HID accessory support), not as a fabricated replacement for macOS display infrastructure.
