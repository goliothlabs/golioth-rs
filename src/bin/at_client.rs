#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_nrf::uarte;
use golioth_rs as _;
use nrf_modem::{send_at_bytes, ConnectionPreference, SystemMode};

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

    run().await;
}

async fn run() -> ! {
    // Handle for device peripherals
    let p = embassy_nrf::init(Default::default());

    // Get uarte default config: Parity::EXCLUDED, Baudrate::BAUD115200
    let config = uarte::Config::default();

    // Setup uarte interrupt and intialize UARTE with configuration
    // Device:  UART RX:  UART TX:  UARTE:
    // Stratus   P0_05     P0_06      0
    // Icarus    P0_06     P0_09      0
    // Thingy    P0_19     P0_18      0
    // 91 DK     P0_??     P0_??      0
    let irq = interrupt::take!(UARTE0_SPIM0_SPIS0_TWIM0_TWIS0);
    let uart = uarte::Uarte::new(p.UARTETWISPI0, irq, p.P0_19, p.P0_18, config);
    let (mut tx, mut rx) = uart.split_with_idle(p.TIMER0, p.PPI_CH0, p.PPI_CH1);

    // Initialize cellular modem with system mode options
    nrf_modem::init(SystemMode {
        lte_support: true,
        lte_psm_support: false,
        nbiot_support: true,
        gnss_support: false,
        preference: ConnectionPreference::Lte,
    })
    .await
    .unwrap();

    let mut buffer = [0; 1024];

    loop {
        // read the command from LTE Link Monitor GUI
        let length = rx.read_until_idle(&mut buffer).await.unwrap();

        // Attempt to send AT command only when rx actually gets some bytes
        if length != 0 {
            let response = send_at_bytes::<1024>(&buffer[..]).await.unwrap();
            tx.write(&response[..].as_bytes()).await.unwrap();
        }
    }
}
