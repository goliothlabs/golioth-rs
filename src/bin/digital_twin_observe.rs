#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m::peripheral::NVIC;
use defmt::{error, info, unwrap, Format};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::{interrupt, pac};
use embassy_time::{Duration, Ticker, Timer};
use golioth_rs::errors::Error;
use golioth_rs::LightDBType::State;
use golioth_rs::*;
use nrf_modem::{ConnectionPreference, SystemMode};
use serde::{Deserialize, Serialize};

// Stucture to hold sensor data: temperature in F, battery level in mV
#[derive(Format, Serialize, Deserialize)]
struct Led {
    blue: bool,
    desired: bool,
}

// Embassy main, where tasks can be spawned
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize heap data for allocation
    info!("Initialize heap");
    heap::init();

    // Run the sample program, will not return unless an error occurs
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
    // Handle for device peripherals
    let p = embassy_nrf::init(Default::default());
    let mut cp = unwrap!(cortex_m::Peripherals::take());

    // Enable the modem interrupts
    info!("Setting up interrupts");
    unsafe {
        NVIC::unmask(pac::Interrupt::EGU1);
        NVIC::unmask(pac::Interrupt::IPC);
        cp.NVIC.set_priority(pac::Interrupt::EGU1, 4 << 5);
        cp.NVIC.set_priority(pac::Interrupt::IPC, 0 << 5);
    }
    // P0_03 -> Blue LED on Conexio Stratus Dev Kit
    // P0_12 -> Blue LED on Actinius Icarus Dev Kit
    // P0_31 -> Blue LED on Thingy 91
    let mut blue = Output::new(p.P0_03, Level::High, OutputDrive::Standard);

    // Structure for the LED's state
    let mut led = Led {
        blue: false,
        desired: true,
    };

    // Create our sleep timer (time between operations)
    let mut ticker = Ticker::every(Duration::from_secs(15));

    // Initialize cellular modem
    unwrap!(
        nrf_modem::init(SystemMode {
            lte_support: true,
            lte_psm_support: false,
            nbiot_support: false,
            gnss_support: false,
            preference: ConnectionPreference::Lte,
        })
        .await
    );

    // Place PSK authentication items in modem for DTLS
    info!("Uploading PSK ID and Key");
    keys::install_psk_id_and_psk().await?;

    // Structure holding our DTLS socket to Golioth Cloud
    info!("Creating DTLS Socket to golioth.io");
    let mut golioth = Golioth::new().await?;

    // Make sure the cloud has a state instance
    info!("Writing to LightDB State");
    golioth.lightdb_write(State, "led", &led).await?;

    todo!();

    // wait for next tick event with low power sleep
    info!("Go to sleep");
    ticker.next().await;


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