use alloc::{format, string::String};
use defmt::{panic, Format, Debug2Format};
use nrfxlib::{AtError, Error as NrfError, at::{self, AtSocket}};

#[derive(Clone, Copy, Format)]
#[allow(dead_code)]
enum Type {
    RootCert = 0,
    ClientCert = 1,
    ClientPrivateKey = 2,
    Psk = 3,
    PskId = 4,
    // ...
}

fn with_cme<R>(f: impl FnOnce(&mut AtSocket) -> Result<R, NrfError>) -> Result<R, NrfError> {
    let mut sock = AtSocket::new()?;
    sock.send_command("AT+CMEE=1\r\n")?;
    sock.poll_response(|_| {})?;
    
    let ret = f(&mut sock)?;
    
    sock.send_command("AT+CMEE=1\r\n")?;
    sock.poll_response(|_| {})?;

    Ok(ret)
}

fn key_delete(tag: u32, ty: Type) -> Result<(), NrfError> {
    with_cme(|sock| {
        let cmd = format!("AT%CMNG=3,{tag},{ty}\r\n", tag = tag, ty = ty as u32);

        sock.send_command(&cmd)?;
        match sock.poll_response(|_| {}) {
            Ok(_) | Err(NrfError::AtError(AtError::CmeError(513))) => Ok(()), // 513 is not found
            e @ Err(_) => e,
        }
    })
}

fn key_write(tag: u32, ty: Type, data: &str) -> Result<(), NrfError> {
    with_cme(|sock| {
        let cmd = format!("AT%CMNG=0,{tag},{ty},\"{data}\"\r\n", tag = tag, ty = ty as u32, data = data);

        sock.send_command(&cmd)?;
        sock.poll_response(|_| {})
    })
}

pub fn install_psk_and_psk_id(security_tag: u32, psk_id: &str, psk: &[u8]) {
    assert!(!psk_id.is_empty() && !psk.is_empty(), "PSK ID and PSK must not be empty. Set them in the `config` module.");

    key_delete(security_tag, Type::PskId).unwrap();
    key_delete(security_tag, Type::Psk).unwrap();

    key_write(security_tag, Type::PskId, psk_id).unwrap();
    key_write(security_tag, Type::Psk, &encode_psk_as_hex(psk)).unwrap();
}

fn encode_psk_as_hex(psk: &[u8]) -> String {
    fn hex_from_digit(num: u8) -> char {
        if num < 10 {
            (b'0' + num) as char
        } else {
            (b'a' + num - 10) as char
        }
    }

    let mut s = String::with_capacity(psk.len() * 2);
    for ch in psk {
        s.push(hex_from_digit(ch / 16));
        s.push(hex_from_digit(ch % 16));
    }

    s
}
