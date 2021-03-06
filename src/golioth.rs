use alloc::{format, vec::Vec};
use coap_lite::{CoapRequest, ContentFormat, Packet, RequestType, error::MessageError};
use core::{str, sync::atomic::{AtomicU16, Ordering}};
use nrfxlib::dtls::{DtlsSocket, PeerVerification, Version};
use serde::{
    de::DeserializeOwned,
    Serialize,
};

use crate::config;

#[derive(Debug)]
pub enum Error {
    Nrf(nrfxlib::Error),
    Coap(MessageError),
    Json(serde_json::error::Error),
}

impl From<nrfxlib::Error> for Error {
    fn from(e: nrfxlib::Error) -> Self {
        Self::Nrf(e)
    }
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

static MESSAGE_ID_COUNTER: AtomicU16 = AtomicU16::new(0);

pub struct Golioth {
    socket: DtlsSocket,
}

impl Golioth {
    pub fn new() -> Result<Self, Error> {
        let socket = DtlsSocket::new(
            PeerVerification::Enabled,
            &[config::SECURITY_TAG],
            Version::Dtls1v2,
        )?;

        socket.connect(config::GOLIOTH_SERVER_URL, config::GOLIOTH_SERVER_PORT)?;

        Ok(Self { socket })
    }

    #[inline]
    fn make_request_and_recv(&mut self, data: &[u8]) -> Result<heapless::Vec<u8, 1024>, Error> {
        self.socket.write(data)?;

        let mut buf = heapless::Vec::new();
        unsafe {
            buf.set_len(1024);
        }

        let read = self.socket.recv_wait(&mut buf[..])?;

        unsafe {
            buf.set_len(read);
        }

        Ok(buf)
    }

    fn lightdb_get_raw(&mut self, path: &str) -> Result<Vec<u8>, Error> {
        let mut request: CoapRequest<()> = CoapRequest::new();

        request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        request.set_method(RequestType::Get);
        request.set_path(&format!(".d/{}", path));
        request
            .message
            .set_content_format(ContentFormat::ApplicationJSON);

        let resp = self.make_request_and_recv(&request.message.to_bytes()?)?;

        let packet = Packet::from_bytes(&resp)?;

        Ok(packet.payload)
    }

    pub fn lightdb_get<T: DeserializeOwned>(&mut self, path: &str) -> Result<T, Error> {
        let payload = self.lightdb_get_raw(path)?;

        Ok(serde_json::from_slice(&payload)?)
    }

    pub fn lightdb_set<T: Serialize>(&mut self, path: &str, v: T) -> Result<(), Error> {
        let mut request: CoapRequest<()> = CoapRequest::new();

        request.message.header.message_id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        request.set_method(RequestType::Post);
        request.set_path(&format!(".d/{}", path));
        request
            .message
            .set_content_format(ContentFormat::ApplicationJSON);
        request.message.payload = serde_json::to_vec(&v)?;

        self.socket.write(&request.message.to_bytes()?)?;

        Ok(())
    }
}
