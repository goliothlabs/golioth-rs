#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use defmt::{info, unwrap,Debug2Format};
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, buffered_uarte, interrupt, pac, peripherals, uarte};
use embassy_nrf::buffered_uarte::BufferedUarte;
use embedded_io_async::Write;
use golioth_rs as _;
use nrf_modem::{send_at_bytes, ConnectionPreference, SystemMode};

bind_interrupts!(struct Irqs {
    UARTE0_SPIM0_SPIS0_TWIM0_TWIS0 => buffered_uarte::InterruptHandler<peripherals::SERIAL0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    run().await;
}

async fn run() -> ! {
    // Handle for device peripherals
    let p = embassy_nrf::init(Default::default());
    let mut cp = unwrap!(cortex_m::Peripherals::take());

    // Enable the modem interrupts
    unsafe {
        NVIC::unmask(pac::Interrupt::IPC);
        cp.NVIC.set_priority(pac::Interrupt::IPC, 0 << 5);
    }

    // Get uarte default config: Parity::EXCLUDED, Baudrate::BAUD115200
    let config = uarte::Config::default();

    // Setup uarte interrupt and intialize UARTE with configuration
    // Device:  UART RX:  UART TX:  UARTE:
    // Stratus   P0_05     P0_06      0
    // Icarus    P0_06     P0_09      0
    // Thingy    P0_19     P0_18      0
    // 91 DK     P0_??     P0_??      0
    // let uart = uarte::Uarte::new(p.SERIAL0, Irqs, p.P0_05, p.P0_06, config);
    let mut tx_buffer = [0u8; 128];
    let mut rx_buffer = [0u8; 128];

    let mut uart = BufferedUarte::new(
        p.SERIAL0,
        p.TIMER0,
        p.PPI_CH0,
        p.PPI_CH1,
        p.PPI_GROUP0,
        Irqs,
        p.P0_05,
        p.P0_06,
        config,
        &mut rx_buffer,
        &mut tx_buffer,
    );

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

    loop {
        let buf = unwrap!(uart.fill_buf().await);
        let n = buf.len();
        info!("command: {}",Debug2Format(&core::str::from_utf8(buf)));
        if buf.contains(&b"\n"[0]) {
            let response = unwrap!(send_at_bytes::<64>(&buf).await);
            unwrap!(uart.write_all(&response[..].as_bytes()).await);
            uart.consume(n);
        }
    }
    let command = [0;64];
    // loop {
    //     let buf = unwrap!(uart.fill_buf().await);
    //     info!("command: {}",Debug2Format(&core::str::from_utf8(buf)));
    //     match buf.find('\n') {
    //         Some(i) => {
    //             let response = unwrap!(send_at_bytes::<64>(&buf).await);
    //             unwrap!(uart.write_all(&response[..].as_bytes()).await);
    //             uart.consume(buf.len());
    //         }
    //         None => {}
    //     }
    // }
}

// Interrupt Handler for LTE related hardware. Defer straight to the library.
#[interrupt]
#[allow(non_snake_case)]
fn IPC() {
    nrf_modem::ipc_irq_handler();
    cortex_m::asm::sev();
}
