extern crate ws;
use ws::{util::Token, CloseCode, Error, ErrorKind, Handshake, Sender};

use crate::message;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::sync::mpsc;

pub struct Server {
    pub clients: HashMap<Token, Sender>,
    pub channels: HashMap<String, BTreeSet<Token>>,
    pub sender: mpsc::Sender<message::PubSubMessage>,
    pub receiver: mpsc::Receiver<message::PubSubMessage>,
}

impl Server {
    pub fn new() -> Server {
        let (tx, rx): (mpsc::Sender<message::PubSubMessage>, mpsc::Receiver<message::PubSubMessage>) = mpsc::channel();
        Server {
            clients: HashMap::new(),
            channels: HashMap::new(),
            sender: tx,
            receiver: rx,
        }
    }

    pub fn add_client(&mut self, client: &Sender) {
        log::debug!("add client: {:?}", client.token());
        let client_token = client.token().clone();
        self.clients.insert(client_token, client.clone());
    }

    pub fn remove_client(&mut self, token: &Token) {
        log::debug!("remove client: {:?}", token);
        self.clients.remove(token);
        // would it be more efficient to leave these tokens in each channel and remove them
        // on the next pub message once we see that the client has been removed?
        for subbed_clients in self.channels.values_mut() {
            subbed_clients.remove(token);
        }
    }

    pub fn sub_client(&mut self, client: &Token, channel: String) -> Result<(), Error> {
        log::debug!("sub client: {:?}", client);
        let subbed_clients = self
            .channels
            .entry(channel.clone())
            .or_insert(BTreeSet::new());
        let result = subbed_clients.insert(client.clone());

        return if result {
            Ok(())
        } else {
            Err(Error {
                kind: ErrorKind::Internal,
                details: Cow::Borrowed("The client was already subscribed to this channel."),
            })
        };
    }

    pub fn unsub_client(&mut self, client: &Token, channel: String) -> Result<(), Error> {
        log::debug!("unsub client: {:?} ", client);
        // Illuminating distinction about how Rust works, the methods above needed owned vars - these need refs.
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            subbed_clients.remove(client);
            return Ok(());
        } else {
            Err(Error {
                kind: ErrorKind::Internal,
                details: Cow::Borrowed("The client was never subscribed to this channel."),
            })
        }
    }

    fn _recieve_msg(&mut self, msg: message::PubSubMessage) -> Result<(), Error> {
        self.sender.send(msg).unwrap();
        return Ok(())
    }

    pub fn pub_message(&mut self, channel: String, msg: &[u8]) -> Result<(), Error> {
        log::debug!("pub message {}", String::from_utf8(msg.to_owned()).unwrap());
        // Probably bad practice to pass an owned variable into the function but use its reference, will come back
        if let Some(subbed_clients) = self.channels.get_mut(&channel) {
            for client_token in subbed_clients.iter() {
                if let Some(client) = self.clients.get(client_token) {
                    client.send(msg.clone())?;
                }
            }

            Ok(())
        } else {
            Ok(())
        }
    }
}

pub struct ClientHandle {
    pub client: Sender,
    pub ws_server_ref: Arc<RefCell<Server>>,
}

impl ClientHandle {
    fn parse_message(&self, raw_msg: &[u8]) -> message::PubSubMessage {
        log::debug!(
            "{:?} parse_message {:?}",
            self.token(),
            String::from_utf8(raw_msg.to_owned()).unwrap()
        );
        let msg = match message::from_slice(raw_msg) {
            Ok(ret) => ret,
            Err(err) => {
                // FIXME: err in rust
                message::PubSubMessage::ERROR {
                    msg: "serde error".to_owned(),
                }
            }
        };
        msg
    }

    fn send(&self, msg: &message::PubSubMessage) -> ws::Result<()> {
        log::debug!("{:?} {:?}", self.token(), msg);
        self.client.send(message::to_string(msg).unwrap())
    }

    fn ok(&self) -> ws::Result<()> {
        self.send(&message::PubSubMessage::OK {})
    }

    fn token(&self) -> Token {
        self.client.token()
    }
}

impl ws::Handler for ClientHandle {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        self.ws_server_ref.borrow_mut().add_client(&self.client);
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        self.ws_server_ref.borrow_mut().remove_client(&self.token());
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        log::debug!("{:?} recieve msg: {:?}", self.token(), msg);
        let msgs = msg.into_data();
        return match self.parse_message(&msgs) {
            message::PubSubMessage::SUBSCRIBE { channel } => {
                self.ws_server_ref
                    .borrow_mut()
                    .sub_client(&self.token(), channel)
                    .unwrap();
                self.ok()
            }
            message::PubSubMessage::UNSUBSCRIBE { channel } => self
                .ws_server_ref
                .borrow_mut()
                .unsub_client(&self.token(), channel),
            message::PubSubMessage::PUBLISH { channel, msg } => {
                self.ws_server_ref.borrow_mut().pub_message(channel, &msg)
            }
            message::PubSubMessage::ERROR { msg } => self.client.send(msg),
            message::PubSubMessage::OK {} => {
                log::trace!("{:?} ignore ok msg", self.token());
                Ok(())
            }
        };
    }
}
