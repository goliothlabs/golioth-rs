#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use defmt_rtt as _; // global logger
use panic_probe as _;
use tinyrlibc as _;

pub mod config;
pub mod heap;
pub mod keys;
pub mod utils;

use crate::config::{GOLIOTH_SERVER_PORT, GOLIOTH_SERVER_URL, SECURITY_TAG};
use at_commands::parser::ParseError;
use coap_lite::{error::MessageError, CoapRequest, ContentFormat, Packet, RequestType};
use core::{
    str,
    sync::atomic::{AtomicU16, Ordering},
};
use defmt::info;
use embassy_time::TimeoutError;
use nrf_modem::{DtlsSocket, PeerVerification};
// use serde::{de::DeserializeOwned, Serialize};

/// Once flashed, comment this out along with the SPM entry in memory.x to eliminate flashing the SPM
/// more than once, and will speed up subsequent builds.  Or leave it and flash it every time
#[link_section = ".spm"]
#[used]
static SPM: [u8; 24052] = *include_bytes!("zephyr.bin");

/// Crate error types
#[derive(Debug)]
pub enum Error {
    Coap(MessageError),
    Json(serde_json::error::Error),
    NrfModem(nrf_modem::Error),
    Timeout(TimeoutError),
    ParseError(ParseError),
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

impl From<TimeoutError> for Error {
    fn from(e: TimeoutError) -> Self {
        Self::Timeout(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
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

        info!("DTLS Socket connected");

        Ok(Self { socket })
    }

    #[inline]
    async fn make_request_and_recv(
        &mut self,
        data: &[u8],
    ) -> Result<(), Error> {
        self.socket.send(data).await?;

        let mut buf = heapless::Vec::<u8, 1024>::new();
        // unsafe { buf.set_len(1024) }

        let (response, _src_addr) = self.socket.receive_from(&mut buf[..]).await?;

        info!("{}", response);

        Ok(())
    }

    // fn lightdb_get_raw(&mut self, path: &str) -> Result<Vec<u8>, Error> {
    //     let mut request: CoapRequest<()> = CoapRequest::new();
    //
    //     request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    //     request.set_method(RequestType::Get);
    //     request.set_path(&format!(".d/{}", path));
    //     request
    //         .message
    //         .set_content_format(ContentFormat::ApplicationJSON);
    //
    //     let resp = self.make_request_and_recv(&request.message.to_bytes()?)?;
    //
    //     let packet = Packet::from_bytes(&resp)?;
    //
    //     Ok(packet.payload)
    // }
    //
    // pub fn lightdb_get<T: DeserializeOwned>(&mut self, path: &str) -> Result<T, Error> {
    //     let payload = self.lightdb_get_raw(path)?;
    //
    //     Ok(serde_json::from_slice(&payload)?)
    // }
    //
    // pub fn lightdb_set<T: Serialize>(&mut self, path: &str, v: T) -> Result<(), Error> {
    //     let mut request: CoapRequest<()> = CoapRequest::new();
    //
    //     request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    //     request.set_method(RequestType::Post);
    //     request.set_path(&format!(".d/{}", path));
    //     request
    //         .message
    //         .set_content_format(ContentFormat::ApplicationJSON);
    //     request.message.payload = serde_json::to_vec(&v)?;
    //
    //     self.socket.write(&request.message.to_bytes()?)?;
    //
    //     Ok(())
    // }
}
