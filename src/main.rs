mod prompb;

use async_trait::async_trait;
use bytes::Bytes;
use dyn_clone::{clone_trait_object, DynClone};
use prompb::remote::WriteRequest;
use prompb::types::TimeSeries;
use protobuf::Message;
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use structopt::StructOpt;
use tokio::task::JoinHandle;
use tracing::{error, info};
use warp::Filter;

#[async_trait]
pub trait Writer: DynClone + Send + Sync {
    async fn write(&self, write_request: WriteRequest);
    fn name(&self) -> String;
}

clone_trait_object!(Writer);

#[derive(Clone)]
struct Graphite {
    address: String,
}

impl Graphite {
    pub fn new(address: String) -> Self {
        Graphite { address }
    }

    fn path_from_metrics(&self, time_series: TimeSeries) -> String {
        let mut path = "".to_string();

        for label in time_series.get_labels() {
            if label.get_name() == "__name__" {
                path.push_str(label.get_value());
            }
        }

        for label in time_series.get_labels() {
            if label.get_name() == "__name__" {
                continue;
            }

            let f = format!(".{}.{}", label.get_name(), label.get_value());

            path.push_str(&f);
        }

        path
    }
}

#[async_trait]
impl Writer for Graphite {
    async fn write(&self, write_request: WriteRequest) {
        let mut r = "".to_string();

        for ts in write_request.timeseries {
            let k = self.path_from_metrics(ts.clone());

            for sample in ts.get_samples() {
                let t = sample.get_timestamp();
                let v = sample.get_value();

                let f = format!("{} {} {}\n", k, v, t);

                r.push_str(&f);
            }
        }

        match TcpStream::connect(&self.address) {
            Ok(mut stream) => match stream.write(r.as_bytes()) {
                Ok(_) => {
                    info!("data written");
                }
                Err(e) => {
                    error!("error: {}", e);
                }
            },
            Err(e) => {
                error!("error: {}", e);
            }
        }
    }

    fn name(&self) -> String {
        "Graphite".to_string()
    }
}

#[derive(Debug, StructOpt, Clone)]
struct Opt {
    /// Where to receive remote write metrics
    #[structopt(short, long, default_value = "3030")]
    ingest_port: u16,

    /// The host:port of the Graphite server to send samples to. None, if empty.
    #[structopt(short, long, default_value = "")]
    graphite_address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let opt = Opt::from_args();

    let ingest_port = opt.ingest_port.clone();

    let mut writers: Vec<Box<dyn Writer>> = vec![];

    if opt.graphite_address != "" {
        let w = Box::new(Graphite::new(opt.graphite_address));

        writers.push(w);
    }

    let writers_filter = warp::any().map(move || writers.clone());

    let write = warp::path!("write")
        .and(warp::body::bytes())
        .and(writers_filter)
        .and_then(handle_write);

    warp::serve(write).run(([127, 0, 0, 1], ingest_port)).await;

    Ok(())
}

async fn handle_write(
    bytes: Bytes,
    writers: Vec<Box<dyn Writer>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match snap::raw::Decoder::new().decompress_vec(&bytes) {
        Ok(buffer) => match WriteRequest::parse_from_bytes(&buffer) {
            Ok(write_request) => {
                let mut workers: Vec<JoinHandle<()>> = Vec::with_capacity(writers.len());

                for writer in writers {
                    let wr = write_request.clone();

                    workers.push(tokio::spawn(async move { writer.write(wr).await }));
                }

                futures::future::join_all(workers).await;

                Ok(warp::reply())
            }
            Err(_) => return Err(warp::reject::reject()),
        },
        Err(_) => return Err(warp::reject::reject()),
    }
}
