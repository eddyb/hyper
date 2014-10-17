#![feature(macro_rules)]
extern crate curl;
extern crate http;
extern crate hyper;

extern crate test;

use std::fmt::{mod, Show};
use std::io::net::ip::Ipv4Addr;
use hyper::server::{Request, Response, Server};
use hyper::method::Method::Get;
use hyper::header::Headers;
use hyper::Client;
use hyper::client::RequestBuilder;

fn listen() -> hyper::server::Listening {
    let server = Server::http(Ipv4Addr(127, 0, 0, 1), 0);
    server.listen(handle).unwrap()
}

macro_rules! try_return(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(..) => return
        }
    }})

fn handle(_r: Request, res: Response) {
    static BODY: &'static [u8] = b"Benchmarking hyper vs others!";
    let mut res = try_return!(res.start());
    try_return!(res.write(BODY))
    try_return!(res.end());
}


#[bench]
fn bench_curl(b: &mut test::Bencher) {
    let mut listening = listen();
    let s = format!("http://{}/", listening.socket);
    let url = s.as_slice();
    b.iter(|| {
        curl::http::handle()
            .get(url)
            .header("X-Foo", "Bar")
            .exec()
            .unwrap()
    });
    listening.close().unwrap();
}

#[deriving(Clone)]
struct Foo;

impl hyper::header::Header for Foo {
    fn header_name(_: Option<Foo>) -> &'static str {
        "x-foo"
    }
    fn parse_header(_: &[Vec<u8>]) -> Option<Foo> {
        None
    }
}

impl hyper::header::HeaderFormat for Foo {
    fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        "Bar".fmt(fmt)
    }
}

#[bench]
fn bench_hyper(b: &mut test::Bencher) {
    let mut listening = listen();
    let s = format!("http://{}/", listening.socket);
    let url = s.as_slice();
    let mut client = Client::new();
    let mut headers = Headers::new();
    headers.set(Foo);
    b.iter(|| {
        client.request(RequestBuilder::new(Get, url).header(Foo)).unwrap().read_to_string().unwrap();
    });
    listening.close().unwrap()
}

/*
doesn't handle keep-alive properly...
#[bench]
fn bench_http(b: &mut test::Bencher) {
    let mut listening = listen();
    let s = format!("http://{}/", listening.socket);
    let url = s.as_slice();
    b.iter(|| {
        let mut req: http::client::RequestWriter = http::client::RequestWriter::new(
            http::method::Get,
            hyper::Url::parse(url).unwrap()
        ).unwrap();
        req.headers.extensions.insert("x-foo".to_string(), "Bar".to_string());
        // cant unwrap because Err contains RequestWriter, which does not implement Show
        let mut res = match req.read_response() {
            Ok(res) => res,
            Err((_, ioe)) => panic!("http response failed = {}", ioe)
        };
        res.read_to_string().unwrap();
    });
    listening.close().unwrap()
}
*/
