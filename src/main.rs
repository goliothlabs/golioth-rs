#![no_main]
#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use nrfxlib::{modem, tcp, tls, at};
use nrf9160_hal::pac::{NVIC, interrupt, Interrupt, CorePeripherals};
use core::{str, fmt::Write as _};
use serde::Deserialize;
use defmt::Format;

use tinyrlibc as _;
use defmt_rtt as _; // global logger
use panic_probe as _;

mod config;
mod heap;
mod utils;
mod golioth;
mod keys;

#[interrupt]
fn EGU1() {
    nrfxlib::application_irq_handler();
    cortex_m::asm::sev();
}

#[interrupt]
fn EGU2() {
    nrfxlib::trace_irq_handler();
    cortex_m::asm::sev();
}

#[interrupt]
fn IPC() {
    nrfxlib::ipc_irq_handler();
    cortex_m::asm::sev();
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();

    // Initialize the heap.
    heap::init();

    unsafe {
        NVIC::unmask(Interrupt::EGU1);
        NVIC::unmask(Interrupt::EGU2);
        NVIC::unmask(Interrupt::IPC);

        core.NVIC.set_priority(Interrupt::EGU1, 4 << 5);
        core.NVIC.set_priority(Interrupt::EGU2, 4 << 5);
        core.NVIC.set_priority(Interrupt::IPC, 0 << 5);
    }

    // Workaround for https://infocenter.nordicsemi.com/index.jsp?topic=%2Ferrata_nRF9160_EngA%2FERR%2FnRF9160%2FEngineeringA%2Flatest%2Fanomaly_160_17.html
    unsafe {
		core::ptr::write_volatile(0x4000_5C04 as *mut u32, 0x02);
	}

    defmt::info!("initializing nrfxlib");

    nrfxlib::init().unwrap();
    modem::flight_mode().unwrap();

    keys::install_psk_and_psk_id(config::SECURITY_TAG, config::PSK_ID, config::PSK);
    
    modem::on().unwrap();

    defmt::info!("connecting to lte");

    modem::wait_for_lte().unwrap();

    defmt::info!("connecting to Golioth");

    let mut golioth = golioth::Golioth::new().unwrap();

    #[derive(Format, Deserialize)]
    struct Leds {
        #[serde(rename(deserialize = "0"))]
        led0: bool,
    }

    // This is not a compliant CoAP implementation, so it won't
    // really work if you try to get multiple paths.
    let leds: Leds = golioth.lightdb_get("led").unwrap();

    defmt::info!("leds: {:?}", leds);

    utils::exit()
}


