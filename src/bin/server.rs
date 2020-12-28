extern crate env_logger;
extern crate newmq;

// use newmq::threadpool;

use newmq::server;
use std::cell::RefCell;
use std::env;
use std::sync::Arc;

fn main() {
    env::set_var("RUST_LOG", "newmq=trace,server=trace,threadpool=trace,ws=warn");
    env_logger::init();

    log::info!("starting up");

    // let pool = threadpool::ThreadPool::new(4);
    // pool.execute(move || {});

    let ws_server = Arc::new(RefCell::new(server::Server::new())); // Will provide internal mutability to all of the client handlers

    ws::listen("127.0.0.1:7878", |client| {
        server::ClientHandle {
            client,
            ws_server_ref: ws_server.clone(),
        }
    })
    .unwrap();
}
