// mostly pirated from rotor-http
use std::marker::PhantomData;

use rotor::transports::stream::{Transport, Protocol};
use rotor::transports::StreamSocket;
//use rotor::buffer_util::find_substr;
use rotor::async::Async;

use rotor::transports::{accept, stream};
use mio::tcp::{TcpListener, TcpStream};

use std::io::Write;
//use std::thread;

const HTTP_RES:&'static str = "HTTP/1.1 200 OK
Date: Sun, 22 Nov 2015 01:00:44 GMT
Server: miohack
Connection: $Connection$
Content-Length: $Content-Length$

$Content$";

pub trait Handler<C> {
    fn dummy(_ctx: &mut C) {

    }
}

pub enum Client<C, H: Handler<C>> {
    /// The initial state of a connection.
    Initial,
    /// The state after some headers have been read.
    // ReadHeaders, // TODO(tailhook) 100 Expect?
    // /// Not yet supported.
    Processing(H, PhantomData<*const C>),
    // /// Reading a request body with a fixed size.
    // ///
    // /// The `usize` gives the number of remaining bytes.
    // ReadFixedSize(Request, usize),
    // TODO ReadChunked(Request, usize),
    /// A connection in idle state.
    KeepAlive,
}

impl<C, H: Handler<C>> Protocol<C> for Client<C, H> {
    fn accepted<S: StreamSocket>(_conn: &mut S, _context: &mut C)
        -> Option<Self>
    {
        Some(Client::Initial)
    }
    fn data_received(self, transport: &mut Transport, _: &mut C)
        -> Async<Self, ()>
    {
        use self::Client::*;
        match self {
            Initial | KeepAlive => { //| ReadHeaders  =>

                //thread::sleep_ms(1);

                // don't even bother parsing the body
                let content = String::from("Have a nice day.");
                let cl = content.to_owned().into_bytes().len(); // TODO: stupid clone

                let res = String::from(HTTP_RES)
                    //.replace("$Connection$","Close")
                    .replace("$Content$", &content)
                    .replace("$Content-Length$", &cl.to_string());

                let out = transport.output();
                write!(out, "{}", res);

                Async::Continue(Client::KeepAlive, ())


            }
            _ => unimplemented!()
        }
    }
}

pub type HttpServer<C, R> = accept::Serve<C,
                        TcpListener,
                        stream::Stream<C, TcpStream, Client<C, R>>>;
