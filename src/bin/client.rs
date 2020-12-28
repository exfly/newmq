use newmq::message;
use newmq::threadpool;
use std::env;
use ws::{connect, CloseCode};

fn main() {
    env::set_var("RUST_LOG", "client=trace,threadpool=trace,ws=debug");
    env_logger::init();

    let pool = threadpool::ThreadPool::new(4);

    pool.execute(move || {
        connect("ws://127.0.0.1:7878", |out| {
            out.send(
                message::to_string(&message::PubSubMessage::SUBSCRIBE {
                    channel: "channel01".to_owned(),
                })
                .unwrap(),
            )
            .unwrap();

            out.send(
                message::to_string(&message::PubSubMessage::PUBLISH {
                    channel: "channel01".to_owned(),
                    msg: "hello".as_bytes().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();

            move |msg| {
                println!("Client got message '{}'. ", msg);
                out.send(message::to_string(&message::PubSubMessage::OK {}).unwrap())
            }
        })
        .unwrap()
    });
}
