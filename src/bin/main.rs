#![no_std]
#![no_main]

use esp_println::println;
use test_ir::IrReceiver;

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("PANIC !!");
    loop {}
}

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut remote = IrReceiver::new(peripherals.RMT, peripherals.GPIO5).unwrap();

    loop {
        if let Ok(code) = remote.receive_code() {
            println!("0x{:02X}", code);
        }
    }
}
