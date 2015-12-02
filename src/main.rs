extern crate rotor;
extern crate mio;
extern crate libc;
extern crate net2;

use std::env;
use std::thread;

use net2::TcpBuilder;
use net2::unix::UnixTcpBuilderExt;

use mio::tcp::TcpListener;

mod http;
use http::HttpServer;

struct HelloWorld;
struct ContextData;

trait Context {
}

impl Context for ContextData {
}

impl<C:Context> http::Handler<C> for HelloWorld {
}

// test with: wrk -t4 -c400 -d30s http://127.0.0.1:8888/index.html

fn single_threaded() {
    println!("single threaded");
    let mut event_loop = mio::EventLoop::new().unwrap();
    let mut handler = rotor::Handler::new(ContextData, &mut event_loop);
    handler.add_root(&mut event_loop,
        HttpServer::<_, HelloWorld>::new(
            TcpListener::bind(
                &"127.0.0.1:8888".parse().unwrap()).unwrap(),
            ));
    event_loop.run(&mut handler).unwrap();
}

fn multi_threaded() {
    let threads = env::var("THREADS").unwrap_or("2".to_string())
        .parse().unwrap();
    println!("using {} threads", threads);
    let mut children = Vec::new();

    let tcp = TcpBuilder::new_v4().unwrap();
    tcp.reuse_address(true).unwrap();
    tcp.reuse_port(true).unwrap();
    let addr = "127.0.0.1:8888";
    println!("Listing on {}", addr);
    tcp.bind(&addr).unwrap();
    let listener = tcp.listen(4096).unwrap();
    let listener = TcpListener::from_listener(listener,&"127.0.0.1:8888".parse().unwrap()).unwrap();

    for _ in 0..threads {
        let listener = listener.try_clone().unwrap();

        children.push(thread::spawn(move || {
            let mut event_loop = mio::EventLoop::new().unwrap();
            let mut handler = rotor::Handler::new(ContextData, &mut event_loop);
            handler.add_root(&mut event_loop,
                HttpServer::<_, HelloWorld>::new(listener));
            event_loop.run(&mut handler).unwrap();
        }));
    }

    for child in children {
        child.join().unwrap();
    }
    println!("threads joined");
}
fn main() {
    let use_st:u32 = env::var("ST").unwrap_or("0".to_string())
        .parse().unwrap();

    if use_st == 1 {
        single_threaded();
    } else {
        multi_threaded();
    }
}
