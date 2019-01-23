//! And implementation of the NSQ protocol,
//! Source: https://github.com/benashford/redis-async-rs/blob/master/src/resp.rs

use std::io;
use std::io::Cursor;
use std::iter::Iterator;
use super::error::Error;

use bytes::{Buf, BufMut, BytesMut};
use tokio_io::codec::{Encoder, Decoder};
use std::str;

use response::Message as TypeMessage;

// Header: Size(4-Byte) + FrameType(4-Byte)
const HEADER_LENGTH: usize = 8;

// Frame Types
const FRAME_TYPE_RESPONSE: i32 = 0x00;
const FRAME_TYPE_ERROR: i32 = 0x01;
const FRAME_TYPE_MESSAGE: i32 = 0x02;

const HEARTBEAT: &'static str = "_heartbeat_";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NSQValue {

    /// An Error form nsqd,
    Error(String),

    /// An Command that not implement sending msgs.
    Command(String),

    /// Succefull response from nsqd.
    Response(String),

    /// Message from nsqd.
    MuxMsg(i64, String, String),

    /// A simple message (pub or dpub).
    Msg(String, String),

    /// Multiple message (mpub)
    MMsg(String, Vec<String>),

    /// nsqd heartbeat msg.
    Heartbeat,
}

impl NSQValue {
    fn into_result(self) -> Result<NSQValue, Error> {
        match self {
            NSQValue::Error(err) => Err(Error::Remote(err)),
            s => Ok(s),
        }
    }
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

fn write_header(buf: &mut BytesMut, size: i32) {
    buf.put_i32_be(size);
}

fn write_cmd(buf: &mut BytesMut, cmd: String) {
    let cmd_as_bytes = cmd.as_bytes();
    let size = cmd_as_bytes.len() + 1;
    check_and_reserve(buf, size);
    buf.extend(cmd_as_bytes);
    write_n(buf);
}

/// write msg:
/// 
/// <command>\n
/// [ 4 byte size in bytes as BigEndian i64 ][ N-byte binary data ]
/// 
/// command could be PUB or DPUB or any command that send a message too.
fn write_msg(buf: &mut BytesMut, cmd: String, msg: String) {
    let cmd_as_bytes = cmd.as_bytes();
    let msg_as_bytes = msg.as_bytes();
    let msg_len = msg_as_bytes.len();
    let size = cmd_as_bytes.len() + 1 + 4 + msg_len;
    check_and_reserve(buf, size);
    buf.extend(cmd_as_bytes);
    write_n(buf);
    buf.put_u32_be(msg_len as u32);
    buf.extend(msg_as_bytes);
}

fn write_mmsg(buf: &mut BytesMut, cmd: String, msgs: Vec<String>) {
    let cmd_as_bytes = cmd.as_bytes();
    let size = cmd_as_bytes.len() + 1;
    check_and_reserve(buf, size);
    buf.extend(cmd_as_bytes);
    write_n(buf);
    for msg in msgs {
        let msg_as_bytes = msg.as_bytes();
        let msg_len = msg_as_bytes.len();
        size = size + 4 + msg_len;
        check_and_reserve(buf, size);
        buf.put_u32_be(msg_len as u32);
        buf.extend(msg_as_bytes);
    }
}


impl Decoder for NsqCodec {
    type Item = NSQValue;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let length = buf.len();

        if length < HEADER_LENGTH {
             return Ok(Some(
                 NSQValue::Error(
                     "Packet lenght must be equal or greater to HEADER_LENGHT".to_owned()
                     )
             ));
        }

        let mut cursor = Cursor::new(buf.clone());
        let size: usize = cursor.get_i32_be() as usize;

        if length < size {
            return Ok(
                Some(
                    NSQValue::Error(
                        "Invalid data size".to_owned()
                    )
                ));
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
                            Ok(Some(NSQValue::Heartbeat))
                        } else {
                            Ok(Some(
                                NSQValue::Response(decoded_message)
                            ))
                        }
                    }
                    Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid UTF-8")),
                }
            },
            FRAME_TYPE_ERROR => {
                Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid packet received"))
            },
            FRAME_TYPE_MESSAGE => {
                let ts = cursor.get_i64_be(); // timestamp
                let _ = cursor.get_u16_be(); // attempts

                let data = str::from_utf8(&cursor.bytes()).unwrap().to_string();
                let (id, msg) = data.split_at(16);

                // remove the serialized frame from the buffer.
                buf.split_to(HEADER_LENGTH + length);

                Ok(Some(
                    NSQValue::MuxMsg(ts, id.to_owned(), msg.to_owned())
                ))              
                },
            _ => {Ok(None)},
        }
    }
}

impl Encoder for NsqCodec {
    type Item = NSQValue;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let ret = match msg {
            NSQValue::Command(cmd) => {
                write_cmd(buf, cmd)
            },
            NSQValue::Msg(cmd, msg) => {
                write_msg(buf, cmd, msg)
            },
            NSQValue::MMsg(cmd, msgs) => {
                write_mmsg(buf, cmd, msgs)
            },
            _ => {},
        };
        Ok(ret)
    }
}

//impl NsqCodec {    
//    fn heartbeat_message(&mut self) -> Frame<String, TypeMessage, io::Error>
//    {
//        let message = TypeMessage{
//            timestamp: 0,
//            message_id: HEARTBEAT.to_string(),
//            message_body: HEARTBEAT.to_string()
//        };
//
//        Frame::Body {
//            chunk: Some(message),
//        } 
//    }
//
//    fn streaming_flag(&mut self) -> Frame<String, TypeMessage, io::Error>
//    {
//        self.decoding_head = false;
//        Frame::Message {
//            message: "".into(),
//            body: true,
//        }
//    }
//}