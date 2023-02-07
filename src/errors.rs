/// Crate error types
#[derive(Debug)]
pub enum Error {
    Coap(coap_lite::error::MessageError),
    Json(serde_json::error::Error),
    NrfModem(nrf_modem::Error),
    Timeout(embassy_time::TimeoutError),
    // ParseError(at_commands::parser::ParseError),
}

impl From<coap_lite::error::MessageError> for Error {
    fn from(e: coap_lite::error::MessageError) -> Self {
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
