use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Serialize, Deserialize)]
pub enum PubSubMessage {
    // For now I won't worry about subbing/unsubbing to an array of channels
    SUBSCRIBE { channel: String },
    UNSUBSCRIBE { channel: String },
    PUBLISH { channel: String, msg: Vec<u8> },
    // PSUBSCRIBE { pattern: String },
    // PUNSUBSCRIBE { pattern: String},
    OK {},
    ERROR { msg: String },
}

pub fn from_slice(v: &[u8]) -> Result<PubSubMessage> {
    let ret: PubSubMessage = serde_json::from_slice(v)?;
    Ok(ret)
}

pub fn to_string(v: &PubSubMessage) -> Result<String> {
    let ret = serde_json::to_string(v)?;
    Ok(ret)
}

#[test]
fn vec_to_message() {
    let s = String::from("s: &str");
    let mut msg = vec![s.len() as u8];
    msg.extend(s.as_bytes().to_owned());
    if let Some((len, datas)) = msg.split_first() {
        let utf8str = String::from_utf8(datas.to_owned());
        println!("len({:?})= {}", utf8str, len);
    }
}
