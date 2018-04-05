extern crate reqwest;
extern crate flate2;
extern crate time;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::process;
use std::env;
use std::time::Duration;
use std::fs::{self, File};
use std::io::BufReader;
use std::io::prelude::*;

use flate2::bufread::GzEncoder;
use flate2::Compression;

use reqwest::StatusCode;
use reqwest::header::{Headers, UserAgent, ContentEncoding, Encoding, Date, HttpDate};

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

    let url = &args[0];
    let path = &args[1];

    let f = File::open(path).map_err(|_| "Could not open ping file")?;
    let reader = BufReader::new(f);

    let level = Compression::new(6); // default compression level
    let mut gz = GzEncoder::new(reader, level);
    let mut buffer = Vec::new();
    gz.read_to_end(&mut buffer).map_err(|_| "Could not read ping file")?;

    let mut headers = Headers::new();
    headers.set(UserAgent(USER_AGENT.to_string()));
    headers.set(ContentEncoding(vec![Encoding::Gzip]));
    headers.set(Date(HttpDate(time::now())));
    headers.set_raw(CUSTOM_VERSION_HEADER, vec![CUSTOM_VERSION.to_vec()]);

    let mut client = reqwest::Client::new().map_err(|_| "Can't create HTTP client")?;
    client.timeout(Duration::from_millis(CONNECTION_TIMEOUT_MS));
    let res = client.post(url)
            .headers(headers)
            .body(buffer)
            .send().map_err(|_| "Could not send HTTP request")?;

    if *res.status() == StatusCode::Ok {
        fs::remove_file(path).map_err(|_| "Could not remove ping file")
    } else {
        Err("Failed to send HTTP request")
    }
}
