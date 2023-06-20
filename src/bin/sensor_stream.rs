#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m::peripheral::NVIC;
use defmt::{error, info, unwrap, Format};
use embassy_executor::Spawner;
use embassy_nrf::{interrupt, pac};

use embassy_time::{Duration, Timer};
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
    let mut cp = unwrap!(cortex_m::Peripherals::take());

    // Enable the modem interrupts
    info!("Setting up interrupts");
    unsafe {
        NVIC::unmask(pac::Interrupt::EGU1);
        NVIC::unmask(pac::Interrupt::IPC);
        cp.NVIC.set_priority(pac::Interrupt::EGU1, 4 << 5);
        cp.NVIC.set_priority(pac::Interrupt::IPC, 0 << 5);
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
    info!("Uploading PSK ID and Key");
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

    // Use LightDB State to record the current state of a sensor
    info!("Writing to LightDB State");
    golioth.lightdb_write(State, write_path, &sensor).await?;

    // send 3 payloads to LightDB Stream
    for _ in 0..3 {
        // Record data to LightDB Stream
        info!("Writing to LightDB Stream");
        golioth.lightdb_write(Stream, write_path, &sensor).await?;

        // Simulate battery drain
        sensor.meta.battery -= 15;
        // get signal strength during transmission
        sensor.meta.signal = get_signal_strength().await?;

        Timer::after(Duration::from_millis(500)).await;
    }

    Ok(())
}

// Interrupt Handler for LTE related hardware. Defer straight to the library.
#[interrupt]
#[allow(non_snake_case)]
fn EGU1() {
    nrf_modem::application_irq_handler();
    cortex_m::asm::sev();
}

// Interrupt Handler for LTE related hardware. Defer straight to the library.
#[interrupt]
#[allow(non_snake_case)]
fn IPC() {
    nrf_modem::ipc_irq_handler();
    cortex_m::asm::sev();
}
