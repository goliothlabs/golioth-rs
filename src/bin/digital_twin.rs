#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{error, info, unwrap, Format};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_time::{Duration, Ticker, Timer};
use futures::StreamExt;
use golioth_rs::errors::Error;
use golioth_rs::LightDBType::{State, Stream};
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

    // Blue LED on Conexio Stratus Dev Kit
    let mut blue = Output::new(p.P0_03, Level::High, OutputDrive::Standard);

    // Structure for the LED's state
    let mut led = Led {
        blue: false,
        desired: false,
    };

    // Create our sleep timer (time between operations)
    // let mut ticker = Ticker::every(Duration::from_secs(30));

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
    // info!("Uploading PSK ID and Key");
    // keys::install_psk_id_and_psk().await?;

    // Structure holding our DTLS socket to Golioth Cloud
    info!("Creating DTLS Socket to golioth.io");
    let mut golioth = Golioth::new().await?;

    let path = "led";

    // Make sure the cloud has a state instance
    info!("Writing to LightDB State");
    golioth.lightdb_write(State, path, &led).await?;

    info!("Writing to LightDB Stream");
    golioth.lightdb_write(Stream, path, &led).await?;

    let digital_twin: Led = golioth.lightdb_read(State,path).await?;
        info!("state read: {}", &digital_twin);

    // loop 3 times
    // for _ in 0..3 {
    //     // Read the state of our device as it exists in the cloud
    //     info!("Reading LightDB State");
    //     let digital_twin: Led = golioth.lightdb_read(State,path).await?;
    //     info!("state read: {}", &digital_twin);
    //
    //     if digital_twin.desired != led.blue {
    //         match digital_twin.desired  {
    //             true => { blue.set_low() }
    //             false => { blue.set_high() }
    //         }
    //         led.blue = digital_twin.desired;
    //         golioth.lightdb_write(State, path, &led).await?;
    //     }
    //
    //     // wait for next tick event with low power sleep
    //     info!("Ticker next()");
    //     // ticker.next().await;
    //     Timer::after(Duration::from_millis(5000)).await;
    // }

    Ok(())
}
