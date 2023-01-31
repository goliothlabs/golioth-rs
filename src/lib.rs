#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate tinyrlibc;

pub mod config;
pub mod heap;
pub mod keys;
pub mod utils;

use alloc::format;
use alloc::vec::Vec;
use defmt_rtt as _; // global logger
use panic_probe as _;
use crate::config::{GOLIOTH_SERVER_PORT, GOLIOTH_SERVER_URL, SECURITY_TAG};
use coap_lite::{error::MessageError, CoapRequest, ContentFormat, Packet, RequestType};
use core::{str};
use defmt::{debug};
use nrf_modem::{DtlsSocket, PeerVerification};
use serde::de::DeserializeOwned;
use serde::{Serialize};


/// Once flashed, comment this out along with the SPM entry in memory.x to eliminate flashing the SPM
/// more than once, and will speed up subsequent builds.  Or leave it and flash it every time
// #[link_section = ".spm"]
// #[used]
// static SPM: [u8; 24052] = *include_bytes!("zephyr.bin");

/// Crate error types
#[derive(Debug)]
pub enum Error {
    Coap(MessageError),
    Json(serde_json::error::Error),
    NrfModem(nrf_modem::Error),
    Timeout(embassy_time::TimeoutError),
    ParseError(at_commands::parser::ParseError),
}

impl From<MessageError> for Error {
    fn from(e: MessageError) -> Self {
        Self::Coap(e)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Self::Json(e)
    }
}

impl From<nrf_modem::Error> for Error {
    fn from(e: nrf_modem::Error) -> Self {
        Self::NrfModem(e)
    }
}

impl From<embassy_time::TimeoutError> for Error {
    fn from(e: embassy_time::TimeoutError) -> Self {
        Self::Timeout(e)
    }
}

impl From<at_commands::parser::ParseError> for Error {
    fn from(e: at_commands::parser::ParseError) -> Self {
        Self::ParseError(e)
    }
}

// Enum for light_db write types
#[derive(Debug)]
pub enum LightDBWriteType {
    State,
    Stream,
}

// static MESSAGE_ID_COUNTER: AtomicU16 = AtomicU16::new(0);

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

    #[inline]
    async fn request_and_recv(&mut self, data: &[u8]) -> Result<heapless::Vec<u8,1024>, Error> {
        self.socket.send(data).await?;

        let mut buf = heapless::Vec::<u8, 1024>::new();

        let (response, _src_addr) = self.socket.receive_from(&mut buf[..]).await?;

        debug!("{}", response);
        let n = response.len();

        buf.truncate(n);

        Ok(buf)
    }

    async fn lightdb_read_raw(&mut self, path: &str) -> Result<Vec<u8>, Error> {
        let mut request: CoapRequest<DtlsSocket> = CoapRequest::new();

        // request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        request.set_method(RequestType::Get);
        request.set_path(&format!(".d/{}", path));
        request
            .message
            .set_content_format(ContentFormat::ApplicationJSON);

        let resp = self.request_and_recv(&request.message.to_bytes()?).await?;

        let packet = Packet::from_bytes(&resp)?;

        Ok(packet.payload)
    }

    pub async fn lightdb_read_state<T: DeserializeOwned>(
        &mut self,
        path: &str,
    ) -> Result<T, Error> {
        let payload = self.lightdb_read_raw(path).await?;

        Ok(serde_json::from_slice(&payload)?)
    }

    pub async fn lightdb_write<T: Serialize>(
        &mut self,
        write_type: LightDBWriteType,
        path: &str,
        v: T,
    ) -> Result<(), Error> {
        let mut request: CoapRequest<DtlsSocket> = CoapRequest::new();

        // request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        request.set_method(RequestType::Post);

        let formatted_path = match write_type {
            LightDBWriteType::State => {
                format!(".d/{}", path)
            }
            LightDBWriteType::Stream => {
                format!(".s/{}", path)
            }
        };
        debug!("set lighdb path: {}", &formatted_path.as_str());

        request.set_path(&formatted_path.as_str());

        request
            .message
            .set_content_format(ContentFormat::ApplicationJSON);
        request.message.payload = serde_json::to_vec(&v)?;
        debug!("sending bytes");
        self.socket.send(&request.message.to_bytes()?).await?;

        Ok(())
    }
}
