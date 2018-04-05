extern crate reqwest;
extern crate flate2;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::process;
use std::env;
use std::fs::File;
use std::io::BufReader;

use flate2::read::GzEncoder;
use flate2::Compression;

use reqwest::{Body, StatusCode};
use reqwest::header::{Headers, UserAgent, ContentEncoding, Encoding};

const USER_AGENT : &str = "pingsender/1.0";
const CUSTOM_VERSION_HEADER : &str = "X-PingSenderVersion";
const CUSTOM_VERSION: &str = "1.0";

fn main() {
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
    let gz = GzEncoder::new(reader, level);

    let mut headers = Headers::new();
    headers.set(UserAgent::new(USER_AGENT));
    headers.set(ContentEncoding(vec![Encoding::Gzip]));
    headers.set_raw(CUSTOM_VERSION_HEADER, CUSTOM_VERSION);

    let client = reqwest::Client::new();
    let res = client.post(url)
            .headers(headers)
            .body(Body::new(gz))
            .send().map_err(|_| "Could not send HTTP request")?;

    if res.status() == StatusCode::Ok {
        Ok(())
    } else {
        Err("Failed to send HTTP request")
    }
}
