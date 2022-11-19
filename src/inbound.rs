use std::{
    io::BufRead,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rocket::{
    http::Status,
    serde::json::{serde_json, Json},
};
use serde::{Deserialize, Serialize};
use std::io::Write;

// Write to file just in case something goes wrong with email
fn store_request(data: String) {
    let logfile = "inbound.log";
    if !Path::new(logfile).exists() {
        std::fs::File::create(logfile).expect("created inbound.log");
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(logfile)
        .unwrap();

    if let Err(e) = writeln!(file, "{}", data,) {
        panic!("Couldn't write to file: {}", e);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InboundRequest {
    pub unix_time: u128,
    pub price: u64,
    pub nodeid: String,
    pub capacity: u64,
    pub duration: u64,
    pub refund_address: String,
    pub payment_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InboundResponse {
    price: u64,
    size: u64,
    duration: u64,
    address: String,
}

#[post("/request-inbound?<nodeid>&<capacity>&<duration>&<refund_address>")]
pub async fn request_inbound(
    nodeid: String,
    capacity: u64,
    duration: u64,
    refund_address: String,
) -> (Status, Json<InboundResponse>) {
    let price = 30_000;
    let resp_capacity = 1_000_000; //sats
    let resp_duration = 1; //month
    let payment_address = crate::lnd::get_onchain_address().await;

    let start = SystemTime::now();
    let unix_time = start.duration_since(UNIX_EPOCH).unwrap().as_millis();

    let channel_request = InboundRequest {
        unix_time,
        price,
        nodeid,
        capacity,
        duration,
        refund_address,
        payment_address: payment_address.clone(),
    };

    let request_str = serde_json::to_string(&channel_request).unwrap();
    store_request(request_str);

    (
        Status::Accepted,
        Json(InboundResponse {
            price,
            size: resp_capacity,
            duration: resp_duration,
            address: payment_address,
        }),
    )
}

pub async fn build_and_send_email(inbound_request: &InboundRequest) {
    let subject = format!("[Channel Request]: {}", inbound_request.nodeid);
    let body = format!(
        "
[Channel Request]
-------------------
NodeID: {}
Capacity: {}
Duration: {}
Payment Address: https://mempool.space/address/{}
Refund Address: {}
-------------------
Cost: {}
        ",
        inbound_request.nodeid,
        inbound_request.capacity,
        inbound_request.duration,
        inbound_request.payment_address,
        inbound_request.refund_address,
        inbound_request.price
    );
    crate::email::send_email(subject, body).await;
}

pub async fn load_inbound_requests() -> Vec<InboundRequest> {
    let logfile = "inbound.log";
    let contents = match std::fs::read(logfile) {
        Err(e) => {
            eprintln!("Unable to open inbound.log file: {:?}", e);
            return vec![];
        }
        Ok(contents) => contents,
    };

    contents
        .lines()
        .filter_map(|line| {
            let request = serde_json::from_str(line.as_ref().expect("valid line"));
            match request {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("Failed to read request from line: {:?}", &line);
                    eprintln!("{:?}", e);
                    None
                }
            }
        })
        .collect::<Vec<_>>()
}
