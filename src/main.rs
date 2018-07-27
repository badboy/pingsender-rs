extern crate hyper;
extern crate hyper_rustls;
extern crate flate2;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::process;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::Cursor;
use std::time::Duration;

use flate2::bufread::GzEncoder;
use flate2::Compression;

use hyper::{Client, Url};
use hyper::header::{Headers, UserAgent, ContentEncoding, Encoding};
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;

const USER_AGENT : &str = "pingsender/1.0";
const CUSTOM_VERSION_HEADER : &str = "X-PingSender-Version";
const CUSTOM_VERSION: &[u8] = b"1.0";
const CONNECTION_TIMEOUT_MS : u64 = 30 * 1000;

fn main() {
    env_logger::init();

    match run() {
        Ok(()) => {},
        Err(e) => {
            warn!("{}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<(), &'static str> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("Usage pingsender URL PATH");
    }

    let url = Url::parse(&args[0]).map_err(|_| "Could not parse URL")?;
    let path = &args[1];

    let f = File::open(path).map_err(|_| "Could not open ping file")?;
    let reader = BufReader::new(f);

    let level = Compression::new(6); // default compression level
    let mut gz = GzEncoder::new(reader, level);
    let mut buffer = Vec::new();
    gz.read_to_end(&mut buffer).map_err(|_| "Could not read ping file")?;

    let mut client = match url.scheme() {
        "http" => Client::new(),
        "https" => {
            let tls = hyper_rustls::TlsClient::new();
            hyper::Client::with_connector(HttpsConnector::new(tls))
        },
        _ => return Err("Unsupported scheme in URL"),
    };

    let mut headers = Headers::new();
    headers.set(UserAgent(USER_AGENT.into()));
    headers.set(ContentEncoding(vec![Encoding::Gzip]));
    headers.set_raw(CUSTOM_VERSION_HEADER, vec![CUSTOM_VERSION.to_vec()]);


    let duration = Duration::from_millis(CONNECTION_TIMEOUT_MS);
    client.set_read_timeout(Some(duration));
    client.set_write_timeout(Some(duration));

    let res = client.post(url)
            .headers(headers)
            .body(&mut Cursor::new(buffer))
            .send().map_err(|_| "Could not send HTTP request")?;

    if res.status == StatusCode::Ok {
        Ok(())
    } else {
        Err("Failed to send HTTP request")
    }
}
