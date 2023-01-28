#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use defmt::{error, Format, info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use golioth_rs::*;
use nrf_modem::{ConnectionPreference, SystemMode};
use serde::{Serialize, Deserialize};
use golioth_rs::config::LOCATION;
use golioth_rs::LightDBWriteType::Stream;

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

    // Initialize heap data
    heap::init();

    // Run our sampling program, will not return unless an error occurs
    match run().await {
        Ok(()) => info!("Program complete!"),
        Err(e) => {
            // If we get here, we have problems
            error!("app exited: {:?}", defmt::Debug2Format(&e));
        }
    }
    // Exit application
    utils::exit();
}

async fn run() -> Result<(), Error> {
    // Handle for device peripherals
    // let mut p = embassy_nrf::init(Default::default());

    // Stucture to hold sensor data: temperature in F, battery level in mV
    #[derive(Format, Serialize, Deserialize)]
    struct TempSensor {
        temp: f32,
        battery: u32,
    }

    // Initialize cellular modem
    unwrap!(
        nrf_modem::init(SystemMode {
            lte_support: true,
            lte_psm_support: true,
            nbiot_support: false,
            gnss_support: false,
            preference: ConnectionPreference::Lte,
        })
        .await
    );

    // Place PSK authentication items in modem for DTLS
    keys::install_psk_id_and_psk().await?;

    // Structure holding our DTLS socket to Golioth Cloud
    info!("Creating DTLS Socket to golioth.io");
    let mut golioth = Golioth::new().await?;

    let mut sensor = TempSensor { temp: 0.0, battery: 0 };

    // Simulate device sensor/adc measurements
    info!("Simulating sensor measurements");
    sensor.temp = 67.5;
    sensor.battery = 3300;

    // Send our data to the cloud
    info!("Sending payload");
    golioth.lightdb_write(Stream, LOCATION, &sensor).await?;

    Ok(())
}
