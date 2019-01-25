//! And implementation of the NSQ protocol,
//! Source: https://github.com/benashford/redis-async-rs/blob/master/src/resp.rs

use std::io::{self, Cursor, Error as IOError, ErrorKind};
use super::error::Error;

use bytes::{Buf, BufMut, BytesMut};
use tokio_io::codec::{Encoder, Decoder};
use std::str;
use std::time::Duration;

// Header: Size(4-Byte) + FrameType(4-Byte)
const HEADER_LENGTH: usize = 8;

// Frame Types
const FRAME_TYPE_RESPONSE: i32 = 0x00;
const FRAME_TYPE_ERROR: i32 = 0x01;
const FRAME_TYPE_MESSAGE: i32 = 0x02;

const HEARTBEAT: &'static str = "_heartbeat_";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NsqValue {

    /// Succefull response.
    Response(String),

    /// Message response.
    ResponseMsg(Duration, String, String),

    /// A simple Command whitch not sends msg.
    Command(String),

    /// A simple message (pub or dpub).
    Msg(String, String),

    /// Multiple message (mpub)
    MMsg(String, Vec<String>),

    /// nsqd heartbeat msg.
    Heartbeat,
}

/// NSQ codec
pub struct NsqCodec;

fn write_n(buf: &mut BytesMut) {
    buf.put_u8(b'\n');
}

fn check_and_reserve(buf: &mut BytesMut, size: usize) {
    let remaining_bytes = buf.remaining_mut();
    if remaining_bytes < size {
        buf.reserve(size);
    }
}

/// write command in buffer and append 0x2 ("\n")
fn write_cmd(buf: &mut BytesMut, cmd: String) {
    let cmd_as_bytes = cmd.as_bytes();
    let size = cmd_as_bytes.len() + 1;
    check_and_reserve(buf, size);
    buf.extend(cmd_as_bytes);
    write_n(buf);
}

/// write command and msg in buffer.
/// 
/// packet format:
/// <command>\n
/// [ 4 byte size in bytes as BigEndian i64 ][ N-byte binary data ]
/// 
/// https://nsq.io/clients/tcp_protocol_spec.html.
/// command could be PUB or DPUB or any command witch send a message.
pub fn write_msg(buf: &mut BytesMut, msg: String) {
    let msg_as_bytes = msg.as_bytes();
    let msg_len = msg_as_bytes.len();
    let size = 4 + msg_len;
    check_and_reserve(buf, size);
    buf.put_u32_be(msg_len as u32);
    buf.extend(msg_as_bytes);
}

/// write multiple messages (aka msub command).
pub fn write_mmsg(buf: &mut BytesMut, cmd: String, msgs: Vec<String>) {
    write_cmd(buf, cmd);
    buf.put_u32_be(msgs.len() as u32);
    for msg in msgs {
        write_msg(buf, msg);
    }
}


impl Decoder for NsqCodec {
    type Item = NsqValue;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let length = buf.len();

        if length < HEADER_LENGTH {
             return Err(
                 Error::IO(
                     IOError::new(ErrorKind::Other, "Invalid HEADER_LENGHT")
                     )
             );
        }

        let mut cursor = Cursor::new(buf.clone());
        let size = cursor.get_i32_be() as usize;

        if length < size {
            println!("lenght: {}, size: {}", length, size);
            println!("{:#?}", &buf[..]);
            return Err(
                    Error::IO(
                        IOError::new(ErrorKind::Other, "Invalid data size")
                    )
                );
        }

        let frame_type: i32 = cursor.get_i32_be();

        match frame_type {
            FRAME_TYPE_RESPONSE => {
                buf.split_to(HEADER_LENGTH + length);
                match str::from_utf8(&cursor.bytes()) {
                    Ok(s) => {
                        let decoded_message = s.to_string();

                        // is heartbeat
                        if decoded_message == HEARTBEAT {
                            Ok(Some(NsqValue::Heartbeat))
                        } else {
                            Ok(Some(
                                NsqValue::Response(decoded_message)
                            ))
                        }
                    }
                    Err(_) => Err(Error::Internal("Invalid UTF-8 Data".to_owned())),
                }
            },
            FRAME_TYPE_ERROR => {
                buf.split_to(HEADER_LENGTH + length);
                match String::from_utf8(Vec::from(cursor.bytes())) {
                    Ok(s) => {
                        Err(Error::Remote(s))
                    },
                    Err(_) => {
                        Err(Error::Internal("Invalid UTF-8".to_owned()))
                    },   
                }
            },
            FRAME_TYPE_MESSAGE => {
                let ts = cursor.get_u64_be(); // timestamp
                let _ = cursor.get_u16_be(); // attempts

                let data = str::from_utf8(&cursor.bytes()).unwrap().to_string();
                let (id, msg) = data.split_at(16);

                // remove the serialized frame from the buffer.
                buf.split_to(HEADER_LENGTH + length);

                Ok(Some(
                    NsqValue::ResponseMsg(Duration::from_nanos(ts), id.to_owned(), msg.to_owned())
                ))              
            },
            _ => {Ok(None)},
        }
    }
}

impl Encoder for NsqCodec {
    type Item = NsqValue;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let ret = match msg {
            NsqValue::Command(cmd) => {
                write_cmd(buf, cmd)
            },
            NsqValue::Msg(cmd, msg) => {
                write_cmd(buf, cmd);
                write_msg(buf, msg)
            },
            NsqValue::MMsg(cmd, msgs) => {
                write_mmsg(buf, cmd, msgs)
            },
            _ => {},
        };
        Ok(ret)
    }
}