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

use crate::config::{GOLIOTH_SERVER_PORT, GOLIOTH_SERVER_URL, SECURITY_TAG};
use crate::errors::Error;
use alloc::format;
use alloc::string::String;
use at_commands::parser::CommandParser;
use coap_lite::MessageType::NonConfirmable;
use coap_lite::{CoapRequest, ContentFormat, Packet, RequestType};
use core::str;
use core::sync::atomic::{AtomicU16, Ordering};
use defmt::debug;
use defmt_rtt as _;
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

        let mut buf = [0; 1024];
        // send request
        self.socket.send(&request.message.to_bytes()?).await?;

        // receive request response
        let (response, _src_addr) = self.socket.receive_from(&mut buf[..]).await?;
        debug!("response: {:X}", &response);

        let packet = Packet::from_bytes(&response)?;

        Ok(serde_json::from_slice(&packet.payload)?)
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
