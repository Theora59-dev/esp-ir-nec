#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_ir_nec::IrReceiver;
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal::main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    println!("IR NEC Receiver");

    let mut remote = IrReceiver::new(peripherals.RMT, peripherals.GPIO5).unwrap();

    loop {
        if let Ok(code) = remote.receive_code() {
            println!("0x{:02X}", code);
        }
    }
}
