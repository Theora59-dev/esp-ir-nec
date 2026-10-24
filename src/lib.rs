//! Réception et décodage d'une télécommande infrarouge NEC sur ESP32.
//!
//! ```no_run
//! use esp_hal::peripherals::Peripherals;
//! use test_ir::IrReceiver;
//!
//! let peripherals = Peripherals::take().unwrap();
//! let mut remote = IrReceiver::new(peripherals.RMT, peripherals.GPIO5).unwrap();
//!
//! loop {
//!     if let Ok(code) = remote.receive_code() {
//!         esp_println::println!("0x{:02X}", code);
//!     }
//! }
//! ```

#![no_std]

mod nec;
mod receiver;

pub use receiver::{IrError, IrReceiver};
