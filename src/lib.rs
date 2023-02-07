#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate tinyrlibc;

pub mod config;
pub mod heap;
pub mod keys;
pub mod utils;
pub mod errors;

use alloc::format;
use crate::config::{GOLIOTH_SERVER_PORT, GOLIOTH_SERVER_URL, SECURITY_TAG};
use coap_lite::{CoapRequest, ContentFormat, Packet, RequestType};
use core::str;
use core::sync::atomic::{AtomicU16, Ordering};
use defmt::{debug, Debug2Format};
use defmt_rtt as _; // global logger
use nrf_modem::{DtlsSocket, PeerVerification};
use panic_probe as _;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::errors::Error;

// Once flashed, comment this out along with the SPM entry in memory.x to eliminate flashing the SPM
// more than once, and will speed up subsequent builds.  Or leave it and flash it every time
// #[link_section = ".spm"]
// #[used]
// static SPM: [u8; 24052] = *include_bytes!("zephyr.bin");


// Enum for light_db write types
#[derive(Debug)]
pub enum LightDBWriteType {
    State,
    Stream,
}

static MESSAGE_ID_COUNTER: AtomicU16 = AtomicU16::new(0);

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

    // #[inline]
    // async fn request_and_recv(&mut self, data: &[u8]) -> Result<heapless::Vec<u8, 1024>, Error> {
    //     let mut buf = heapless::Vec::<u8, 1024>::new();
    //     self.socket.send(data).await?;
    //
    //     let (response, _src_addr) = self.socket.receive_from(&mut buf[..]).await?;
    //
    //     let n = response.len();
    //
    //     buf.truncate(n);
    //
    //     Ok(buf)
    // }

    pub async fn lightdb_read_state<T: DeserializeOwned>(
        &mut self,
        path: &str,
    ) -> Result<T, Error> {
        let mut request: CoapRequest<DtlsSocket> = CoapRequest::new();

        request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        request.set_method(RequestType::Get);
        request.set_path(&format!(".d/{}", path));
        request
            .message
            .set_content_format(ContentFormat::ApplicationJSON);

        debug!("CoAP read header: {}", Debug2Format(&request.message.header));

        let mut buf = [0;1024];

        self.socket.send(&request.message.to_bytes()?).await?;

        let (response, _src_addr) = self.socket.receive_from(&mut buf[..]).await?;
        debug!("response: {:X}", &response);

        let packet = Packet::from_bytes(&response)?;
        debug!("Packet: {:X}", Debug2Format(&packet.payload));

        Ok(serde_json::from_slice(&packet.payload)?)
    }

    pub async fn lightdb_write<T: Serialize>(
        &mut self,
        write_type: LightDBWriteType,
        path: &str,
        v: T,
    ) -> Result<(), Error> {
        let mut request: CoapRequest<DtlsSocket> = CoapRequest::new();

        request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
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
