extern crate rotor;
extern crate mio;
extern crate libc;

use std::env;
use std::thread;
use std::os::unix::io::AsRawFd;

struct ContextData {
    dummy: u32
}

use mio::tcp::TcpListener;

mod http;

use http::HttpServer;

struct HelloWorld;

trait Context {

}

impl Context for ContextData {

}

impl<C:Context> http::Handler<C> for HelloWorld {
    fn dummy(_ctx: &mut C) {

    }
}

fn single_threaded() {
    println!("single threaded");
    let mut event_loop = mio::EventLoop::new().unwrap();
    let mut handler = rotor::Handler::new(ContextData {
        dummy: 0,
    }, &mut event_loop);
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
    for _ in 0..threads {
        let sock = mio::tcp::TcpSocket::v4().unwrap();
        let one = 1i32;
        //sock.set_reuseport(true).unwrap(); // in mio master, but not in crate version
        //sock.set_reuseaddr(true).unwrap(); // wrong one
        unsafe {
            assert!(libc::setsockopt(
                sock.as_raw_fd(), libc::SOL_SOCKET,
                libc::SO_REUSEPORT,
                &one as *const libc::c_int as *const libc::c_void, 4) == 0);
        }
        sock.bind(&"127.0.0.1:8888".parse().unwrap()).unwrap();
        let listener = sock.listen(4096).unwrap();
        children.push(thread::spawn(move || {
            let mut event_loop = mio::EventLoop::new().unwrap();
            let mut handler = rotor::Handler::new(ContextData {
                dummy: 0,
            }, &mut event_loop);
            handler.add_root(&mut event_loop,
                HttpServer::<_, HelloWorld>::new(listener));
            event_loop.run(&mut handler).unwrap();
        }));
    }
    for child in children {
        child.join().unwrap();
    }
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
