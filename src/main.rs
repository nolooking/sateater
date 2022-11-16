#[macro_use]
extern crate rocket;

pub mod cashu;
pub mod wildcard;
use std::sync::Mutex;

// use clightningrpc::LightningRPC;
use qrcode_generator::QrCodeEcc;
use rocket::{fs::FileServer, http::Status, serde::json::Json};
use sateater::{
    cashu::cashu_receive,
    load_conf,
    vault::{board, land, BattleConfig},
    BOARD_SIZE,
};
use serde::Serialize;

// const RPC_FILE: &str = "lightning-rpc";

// // Database not required
// use rocket_db_pools::{
//     sqlx::{self, Sqlite, Transaction},
//     Database,
// };
// #[derive(Database)]
// #[database("main")]
// pub struct Main(sqlx::SqlitePool);

#[derive(Serialize, Debug)]
pub struct PaymentResponse {
    pub amount: i64,
    pub address: String,
    pub payment_id: String,
    pub message: String,
}

#[get("/create_payment?<amount>&<message>")]
pub async fn create_payment(
    amount: i64,
    message: Option<String>,
) -> (Status, Json<PaymentResponse>) {
    // let node = LightningRPC::new(RPC_FILE);
    // let payment_id = Uuid::new_v4();
    let (address, cert, macaroon, label) = load_conf();
    let mut client = tonic_lnd::connect_lightning(address, cert, macaroon)
        .await
        .expect("failed to connect");

    let description = match message {
        Some(message) => message,
        None => label,
    };

    // let sat_amount = amount.checked_mul(1000).expect("not billions(?) of sats") as i64;

    let invoice = tonic_lnd::lnrpc::Invoice {
        memo: description,
        value: amount,
        ..Default::default()
    };

    let created_invoice = client
        .add_invoice(invoice)
        .await
        // , &payment_id.to_string(), &description, None)
        .expect(
            "created invoice with lightning node using ssh tunnel!\n
             Run `ssh pi@your.node.ip -q -N -L 10009:localhost:10009`\n\n",
            // Run `ssh -nNT -L ./lightning-rpc:/root/.lightning/bitcoin/lightning-rpc user@host`\n\n"
        )
        .into_inner();

    let payment_id = hex::encode(created_invoice.r_hash);
    create_qr_code(
        created_invoice.payment_request.clone(),
        payment_id.to_string(),
    );

    (
        Status::Accepted,
        Json(PaymentResponse {
            amount,
            address: created_invoice.payment_request,
            payment_id: payment_id.to_string(),
            message: "payment created".to_string(),
        }),
    )
}

fn create_qr_code(qr_string: String, payment_id: String) {
    qrcode_generator::to_png_to_file(
        qr_string,
        QrCodeEcc::Low,
        512,
        format!("html/qr_codes/{}.png", payment_id),
    )
    .expect("created qr code OK")
}

#[derive(Serialize, Debug)]
pub struct PaymentStatusResponse {
    payment_complete: bool,
    confirmed_paid: u64,
    unconfirmed_paid: u64,
}

#[get("/check_payment?<payment_id>")]
pub async fn check_payment(payment_id: String) -> (Status, Json<PaymentStatusResponse>) {
    let (address, cert, macaroon, _label) = load_conf();
    let mut client = tonic_lnd::connect_lightning(address, cert, macaroon)
        .await
        .expect("failed to connect");

    let payment_hash = tonic_lnd::lnrpc::PaymentHash {
        r_hash: hex::decode(payment_id).expect("valid payment hash"),
        ..Default::default()
    };

    let invoice = client
        .lookup_invoice(payment_hash)
        .await
        .expect("fetched invoices")
        .into_inner();

    let payment_complete = if invoice.state == 1 { true } else { false };

    let response = PaymentStatusResponse {
        payment_complete: payment_complete,
        // For later doing onchain
        confirmed_paid: 0,
        unconfirmed_paid: 0,
    };
    (Status::Accepted, Json(response))
}

#[get("/receive_ecash?<token>")]
pub async fn receive_ecash(token: String) -> (Status, Json<PaymentStatusResponse>) {
    let amount_paid = cashu_receive(&token);
    let response = PaymentStatusResponse {
        payment_complete: amount_paid > 0,
        // For later doing onchain
        confirmed_paid: amount_paid as u64,
        unconfirmed_paid: 0,
    };
    (Status::Accepted, Json(response))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from("./html"))
        .mount(
            "/api",
            routes![
                create_payment,
                check_payment,
                receive_ecash,
                wildcard::wildcard,
                board,
                land
            ],
        )
        .manage(Mutex::new(BattleConfig {
            board: vec![vec![(0, 0); BOARD_SIZE.into()]; BOARD_SIZE.into()],
        }))
}
