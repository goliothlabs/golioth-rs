#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate tinyrlibc;

pub mod config;
pub mod errors;
pub mod heap;
pub mod keys;
pub mod utils;
// pub mod ffi;

use crate::config::{GOLIOTH_SERVER_PORT, GOLIOTH_SERVER_URL, SECURITY_TAG};
use crate::errors::Error;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use at_commands::parser::CommandParser;
use coap_lite::MessageType::NonConfirmable;
use coap_lite::{CoapRequest, ContentFormat, Packet, RequestType};
use core::str;
use core::sync::atomic::{AtomicU16, Ordering};
use defmt::{debug, Debug2Format};
use defmt_rtt as _;
use embassy_time::{with_timeout, Duration};
use nanorand::{Rng, WyRand};
use nrf_modem::{DtlsSocket, PeerVerification};
use panic_probe as _;
use serde::de::DeserializeOwned;
use serde::Serialize;

// Once flashed, comment this out along with the SPM entry in memory.x to eliminate flashing the SPM
// more than once, and will speed up subsequent builds.  Or leave it and flash it every time
#[link_section = ".spm"]
#[used]
static SPM: [u8; 24052] = *include_bytes!("zephyr.bin");

// use for CoAP mesaage header ID to avoid requests being flagged as duplicate messages
static MESSAGE_ID_COUNTER: AtomicU16 = AtomicU16::new(0);

// Enum for light_db write types
#[derive(Debug)]
pub enum LightDBType {
    State,
    Stream,
}

// Struct to hold our DTLS Socket to Golioth, should live the length of the program
pub struct Golioth {
    socket: DtlsSocket,
}

impl Golioth {
    pub async fn new() -> Result<Self, Error> {
        let socket = DtlsSocket::connect(
            GOLIOTH_SERVER_URL,
            GOLIOTH_SERVER_PORT,
            PeerVerification::Enabled,
            &[SECURITY_TAG],
        )
        .await?;

        debug!("DTLS Socket created");

        Ok(Self { socket })
    }

    async fn handle_response<T: DeserializeOwned>(
        &mut self,
        request: CoapRequest<DtlsSocket>,
    ) -> Result<T, Error> {
        let mut buf = [0; 1024];
        let request_token = request.message.get_token();

        loop {
            // receive response
            let (response, _src_addr) = self.socket.receive_from(&mut buf[..]).await?;
            let packet = Packet::from_bytes(&response)?;
            debug!("Response Bytes: {:X}", &response);
            debug!("Response token: {}", &packet.get_token());

            // make sure the request token matches the response token before returning results
            if packet.get_token() == request_token {
                debug!("Token Match!");
                return Ok(serde_json::from_slice(&packet.payload)?);
            }
        }
    }

    // the DeserializeOwned trait is equivalent to the higher-rank trait bound
    // for<'de> Deserialize<'de>. The only difference is DeserializeOwned is more
    // intuitive to read. It means T owns all the data that gets deserialized.
    pub async fn lightdb_read<T: DeserializeOwned>(
        &mut self,
        read_type: LightDBType,
        path: &str,
    ) -> Result<T, Error> {
        let mut request: CoapRequest<DtlsSocket> = CoapRequest::new();
        let formatted_path = get_formatted_path(read_type, path);

        request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        request.set_method(RequestType::Get);
        request.set_path(&formatted_path);
        request
            .message
            .set_content_format(ContentFormat::ApplicationJSON);
        request.message.set_token(create_token());

        // send request
        self.socket.send(&request.message.to_bytes()?).await?;

        with_timeout(Duration::from_secs(2), self.handle_response(request)).await?
    }

    pub async fn lightdb_write<T: Serialize>(
        &mut self,
        write_type: LightDBType,
        path: &str,
        v: T,
    ) -> Result<(), Error> {
        let mut request: CoapRequest<DtlsSocket> = CoapRequest::new();
        // message header id distinguishes duplicate messages
        request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        request.set_method(RequestType::Post);
        // Do not ask for a confirmed response
        request.message.header.set_type(NonConfirmable);
        request.message.set_token(create_token());

        let formatted_path = get_formatted_path(write_type, path);
        debug!("set lighdb path: {}", &formatted_path.as_str());

        request.set_path(&formatted_path);

        request
            .message
            .set_content_format(ContentFormat::ApplicationJSON);
        request.message.payload = serde_json::to_vec(&v)?;

        debug!("sending bytes");
        self.socket.send(&request.message.to_bytes()?).await?;

        Ok(())
    }
}

#[inline]
fn get_formatted_path(db_type: LightDBType, path: &str) -> String {
    match db_type {
        LightDBType::State => {
            format!(".d/{}", path)
        }
        LightDBType::Stream => {
            format!(".s/{}", path)
        }
    }
}

// The nRF9160 does not have a RNG peripheral.  To bypass using the Cryptocell C-lib
// we can just use the current uptime ticks u64 value as a way to create a unique "enough"
// token as to not be easily spoofed.
fn create_token() -> Vec<u8> {
    let seed = embassy_time::Instant::now().as_ticks();
    let mut rng = WyRand::new_seed(seed);
    let token = rng.generate::<u64>().to_ne_bytes().to_vec();

    debug!("WyRand Request Token: {}", Debug2Format(&token));

    token
}

pub async fn get_signal_strength() -> Result<i32, Error> {
    let command = nrf_modem::send_at::<32>("AT+CESQ").await?;

    let (_, _, _, _, _, mut signal) = CommandParser::parse(command.as_bytes())
        .expect_identifier(b"+CESQ:")
        .expect_int_parameter()
        .expect_int_parameter()
        .expect_int_parameter()
        .expect_int_parameter()
        .expect_int_parameter()
        .expect_int_parameter()
        .expect_identifier(b"\r\n")
        .finish()
        .unwrap();
    if signal != 255 {
        signal += -140;
    }
    Ok(signal)
}
