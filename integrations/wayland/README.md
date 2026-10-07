# Wayland integration

Wayland integration stays compositor-neutral where possible.

The DDC layer is independent of Wayland. This adapter will consume output-management/window
identity protocols when available and degrade gracefully when a compositor does not expose them.
