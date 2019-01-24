#[cfg(test)]
extern crate bytes;
extern crate nsqueue;
extern crate tokio_io;

use bytes::BytesMut;
use nsqueue::{NsqCodec, NsqValue};
use tokio_io::codec::{Encoder, Decoder};

macro_rules! codec_cmd_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, exp) = $value;
                let i = NsqValue::Command(input.to_owned());
                let mut codec = NsqCodec{};
                let mut buf = BytesMut::new();
                if codec.encode(i, &mut buf).is_ok() {
                    assert_eq!(exp, &buf[..]);
                }
            }
        )*
        
    }
}

macro_rules! codec_msg_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (cmd, msg, exp) = $value;
                let i = NsqValue::Msg(cmd.to_owned(), msg.to_owned());
                let mut codec = NsqCodec{};
                let mut buf = BytesMut::new();
                if codec.encode(i, &mut buf).is_ok() {
                    assert_eq!(exp, &buf[..]);
                }
            }
        )*
    }
}

macro_rules! codec_mmsg_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (cmd, msg, exp) = $value;
                let msgs: Vec<String> = msg.into_iter().map(|x| x.to_owned()).collect();
                let i = NsqValue::MMsg(cmd.to_owned(), msgs);
                let mut codec = NsqCodec{};
                let mut buf = BytesMut::new();
                if codec.encode(i, &mut buf).is_ok() {
                    assert_eq!(exp, &buf[..]);
                }
            }
        )*
    }
}

codec_cmd_tests! {
    encode_sub_test: ("SUB topic channel", b"SUB topic channel\n"),
    encode_v2_test: ("  V2", b"  V2\n"),
    encode_nop_test: ("NOP", b"NOP\n"),
    encode_cls_test: ("CLS", b"CLS\n"),
    encode_req_test: ("REQ 03ff", b"REQ 03ff\n"),
    encode_dry_test: ("DRY 0", b"DRY 0\n"),
}

codec_msg_tests! {
    encode_indentify_test: ("IDENTIFY", "{}", &[73, 68, 69, 78, 84, 73, 70, 89, 10, 0, 0, 0, 2, 123, 125]),
    encode_pub_test: ("PUB test", "hello world", &[80, 85, 66, 32, 116, 101, 115, 116, 10, 0, 0, 0, 11, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]),
    encode_pub_utf8_test: ("PUB test", "©¥£", &[80, 85, 66, 32, 116, 101, 115, 116, 10, 0, 0, 0, 6, 194, 169, 194, 165, 194, 163]),
}

codec_mmsg_tests! {
    encode_mpub_test: ("MPUB test", vec!["hello", "world"], &[77, 80, 85, 66, 32, 116, 101, 115, 116, 10, 0, 0, 0, 2, 0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0, 5, 119, 111, 114, 108, 100]),
}