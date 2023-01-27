#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Flex, Level, Output, OutputDrive};
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_time::{with_timeout, Duration, Ticker, Timer};
use golioth_rs::*;
use nrf_modem::{ConnectionPreference, SystemMode};
use serde::{Deserialize, Serialize};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Set up the interrupts for the modem
    let egu1 = interrupt::take!(EGU1);
    egu1.set_priority(Priority::P4);
    egu1.set_handler(|_| {
        nrf_modem::application_irq_handler();
        cortex_m::asm::sev();
    });
    egu1.enable();

    let ipc = interrupt::take!(IPC);
    ipc.set_priority(Priority::P0);
    ipc.set_handler(|_| {
        nrf_modem::ipc_irq_handler();
        cortex_m::asm::sev();
    });
    ipc.enable();

    // Workaround for https://infocenter.nordicsemi.com/index.jsp?topic=%2Ferrata_nRF9160_EngA%2FERR%2FnRF9160%2FEngineeringA%2Flatest%2Fanomaly_160_17.html
    unsafe {
        core::ptr::write_volatile(0x4000_5C04 as *mut u32, 0x02);
    }

    // Initialize heap data
    heap::init();

    // Run our sampling program, will not return unless an error occurs
    match run().await {
        Ok(()) => info!("Program complete!"),
        Err(e) => {
            // If we get here, we have problems
            error!("app exited: {:?}", defmt::Debug2Format(&e));
            utils::exit();
        }
    }
}

async fn run() -> Result<(), Error> {
    // Handle for device peripherals
    let mut p = embassy_nrf::init(Default::default());

    // Initialize cellular modem
    unwrap!(
        nrf_modem::init(SystemMode {
            lte_support: true,
            lte_psm_support: true,
            nbiot_support: true,
            gnss_support: false,
            preference: ConnectionPreference::Lte,
        })
        .await
    );

    keys::install_psk_id_and_psk();

    // let mut ticker = Ticker::every(Duration::from_secs(3));

    Timer::after(Duration::from_micros(500)).await;

    info!("Send payload here");

    // ticker.next().await; // wait for next tick event

    Ok(())
}

/*

fn run(delay: &mut impl DelayMs<u32>) -> Result<(), golioth::Error> {
    let mut golioth = golioth::Golioth::new()?;

    #[derive(Format, Deserialize)]
    struct Leds {
        #[serde(rename(deserialize = "0"))]
        led0: bool,
    }

    let leds: Leds = golioth.lightdb_get("led")?;

    defmt::info!("leds: {:?}", leds);

    #[derive(Serialize)]
    struct Counter {
        i: usize,
    }

    for i in 0.. {
        defmt::info!("writing to /counter");
        golioth.lightdb_set("counter", Counter { i })?;

        delay.delay_ms(5_000);
    }

    Ok(())
}
 */
