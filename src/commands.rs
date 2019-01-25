use codec::NsqValue;

const VERSION: &'static str = "  V";
const PUB: &'static str = "PUB";
const MPUB: &'static str = "MPUB";
const DPUB: &'static str = "DPUB";
const SUB: &'static str = "SUB";
const TOUCH: &'static str = "TOUCH";
const RDY: &'static str = "RDY";
const FIN: &'static str = "FIN";
const CLS: &'static str = "CLS";
const NOP: &'static str = "NOP";
const IDENTIFY: &'static str = "IDENTIFY";
const REQ: &'static str = "REQ";

pub fn nop() -> NsqValue {
    NsqValue::Command(format!("{}", NOP))
}

pub fn identify(config: String) -> NsqValue {
    NsqValue::Msg(format!("{}", IDENTIFY), config)
}

pub fn fin() -> NsqValue {
    NsqValue::Command(format!("{}", FIN))
}

pub fn cls() -> NsqValue {
    NsqValue::Command(format!("{}", CLS))
}

pub fn rdy(i: usize) -> NsqValue {
    NsqValue::Command(format!("{} {}", RDY, i))
}

pub fn version(v: usize) -> NsqValue {
    NsqValue::Command(format!("{}{}", VERSION, v))
}

pub fn touch(id: &str) -> NsqValue {
    NsqValue::Command(format!("{} {}", TOUCH, id))
}

pub fn req(id: &str, timeout: &str) -> NsqValue {
    NsqValue::Command(format!("{} {} {}", REQ, id, timeout))
}

pub fn publish(topic: &str, msg: &str) -> NsqValue {
    NsqValue::Msg(format!("{} {}", PUB, topic), msg.to_owned())
}

pub fn mpub(topic: &str, msgs: Vec<String>) -> NsqValue {
    NsqValue::MMsg(format!("{} {}", MPUB, topic), msgs)
}

pub fn dpub(topic: &str, defer_time: &str, msg: &str) -> NsqValue {
    NsqValue::Msg(format!("{} {} {}", DPUB, topic, defer_time), msg.to_owned())
}
