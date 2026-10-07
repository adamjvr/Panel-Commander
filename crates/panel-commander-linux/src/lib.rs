#![cfg(target_os = "linux")]

pub mod connector_transport;
pub mod discovery;
pub mod i2c;

pub use connector_transport::open_ddc_for_connector;
pub use discovery::{discover_connectors, Connector};
pub use i2c::{I2cDdcTransport, I2cTransferMethod};
