#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{error, info, unwrap, Format};
use embassy_executor::Spawner;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use golioth_rs::errors::Error;
use golioth_rs::LightDBType::{State, Stream};
use golioth_rs::*;
use nrf_modem::{ConnectionPreference, SystemMode};
use serde::{Deserialize, Serialize};

// Structure to hold sensor data: temperature in F, battery level in mV
#[derive(Format, Serialize, Deserialize)]
struct TempSensor {
    temp: f32,
    meta: Meta,
}

#[derive(Format, Serialize, Deserialize)]
pub struct Meta {
    battery: u32,
    signal: i32,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Set up the interrupts for the modem
    info!("Setting up interrupts");
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
    info!("Initialize heap");
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
    info!("starting application");

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
    // info!("Uploading PSK ID and Key");
    // keys::install_psk_id_and_psk().await?;

    // Structure holding our DTLS socket to Golioth Cloud
    info!("Creating DTLS Socket to golioth.io");
    let mut golioth = Golioth::new().await?;

    let mut sensor = TempSensor {
        temp: 0.0,
        meta: Meta {
            battery: 0,
            signal: 0,
        },
    };

    // Simulate device sensor/adc measurements
    info!("Simulating sensor measurements");
    sensor.temp = 67.5;
    sensor.meta.battery = 3300;

    let write_path = "data";

    for _ in 0..1 {
        // Use LightDB State to record the current state of a sensor
        info!("Writing to LightDB State");
        golioth.lightdb_write(State, write_path, &sensor).await?;

        // Record data to LightDB Stream
        info!("Writing to LightDB Stream");
        golioth.lightdb_write(Stream, write_path, &sensor).await?;

        // Simulate battery drain
        sensor.meta.battery -= 15;
        sensor.meta.signal = get_signal_strength().await?;
    }

    let digital_twin: TempSensor = golioth.lightdb_read_state(write_path).await?;
    info!("state read: {}", &digital_twin);

    Ok(())
}
