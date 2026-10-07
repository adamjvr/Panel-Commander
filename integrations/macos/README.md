# macOS integration

The native monitor transport will be implemented in `panel-commander-macos`.

Recovered cross-platform behavior:
- Intel macOS: IOFramebuffer / IOI2CInterface DDC path.
- Apple Silicon: IOAVServiceReadI2C / IOAVServiceWriteI2C path.
- per-transport serialization and address fallback.
- UI/profile logic remains in Rust above the transport.

Any tiny C/Objective-C/C++ shim that proves unavoidable will expose a narrow C ABI and contain no
business logic.
