use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;
use std::time::Duration;

use flate2::bufread::GzEncoder;
use flate2::Compression;
use log::warn;

use chttp::http::Request;
use chttp::options::Options;

const USER_AGENT: &str = "pingsender/1.0";
const CONTENT_ENCODING: &str = "gzip";
const CUSTOM_VERSION_HEADER: &str = "X-PingSender-Version";
const CUSTOM_VERSION: &str = "1.0";
const CONNECTION_TIMEOUT_MS: u64 = 30 * 1000;

fn main() {
    env_logger::init();

    match run() {
        Ok(()) => {}
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
    gz.read_to_end(&mut buffer)
        .map_err(|_| "Could not read ping file")?;

    let client = chttp::Client::builder()
        .options(
            Options::default().with_timeout(Some(Duration::from_millis(CONNECTION_TIMEOUT_MS))),
        )
        .build()
        .map_err(|_| "Could not create HTTP client")?;

        let date = format!("{}", time::now().rfc822());
    let req = Request::post(url)
        .header("User-Agent", USER_AGENT)
        .header("Content-Encoding", CONTENT_ENCODING)
        .header("Date", date)
        .header(CUSTOM_VERSION_HEADER, CUSTOM_VERSION)
        .body(buffer).map_err(|_| "Could not create HTTP request")?;

    let res = client.send(req).map_err(|_| "Could not send HTTP request")?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Failed to send HTTP request")
    }
}
