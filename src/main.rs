extern crate reqwest;
extern crate flate2;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;

use std::process;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::time::Duration;

use flate2::bufread::GzEncoder;
use flate2::Compression;

use reqwest::header::{self, HeaderMap, HeaderValue};

const USER_AGENT : &str = "pingsender/1.0";
const CONTENT_ENCODING : &str = "gzip";
const CUSTOM_VERSION_HEADER : &str = "X-PingSender-Version";
const CUSTOM_VERSION: &str = "1.0";
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

    let url = &args[0];
    let path = &args[1];

    let f = File::open(path).map_err(|_| "Could not open ping file")?;
    let reader = BufReader::new(f);

    let level = Compression::new(6); // default compression level
    let mut gz = GzEncoder::new(reader, level);
    let mut buffer = Vec::new();
    gz.read_to_end(&mut buffer).map_err(|_| "Could not read ping file")?;

    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static(CONTENT_ENCODING));
    let date = format!("{}", time::now().rfc822());
    headers.insert(header::DATE, HeaderValue::from_str(&date).unwrap());
    headers.insert(CUSTOM_VERSION_HEADER, HeaderValue::from_static(CUSTOM_VERSION));

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_millis(CONNECTION_TIMEOUT_MS))
        .build()
        .map_err(|_| "Could not create HTTP client")?;
    let res = client.post(url)
            .headers(headers)
            .body(buffer)
            .send().map_err(|_| "Could not send HTTP request")?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Failed to send HTTP request")
    }
}
