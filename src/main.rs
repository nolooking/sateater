#[macro_use]
extern crate rocket;

use clightningrpc::LightningRPC;
use qrcode_generator::QrCodeEcc;
use rocket::{fs::FileServer, http::Status, serde::json::Json};
use serde::Serialize;
use uuid::Uuid;

const RPC_FILE: &str = "lightning-rpc";
const DEFAULT_LABEL: &str = "thanks!";

// Database not required

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
    let payment_id = Uuid::new_v4();
    let node = LightningRPC::new(RPC_FILE);

    let description = match message {
        Some(message) => message,
        None => DEFAULT_LABEL.to_string(),
    };

    let invoice = node
        .invoice(amount.checked_mul(1000).expect("not billions(?) of sats") as u64, &payment_id.to_string(), &description, None)
        .expect(
            "created invoice with lightning node using ssh tunnel!\n
             Run `ssh -nNT -L ./lightning-rpc:/root/.lightning/bitcoin/lightning-rpc user@host`\n\n",
        );

    create_qr_code(invoice.bolt11.clone(), payment_id.to_string());

    (
        Status::Accepted,
        Json(PaymentResponse {
            amount,
            address: invoice.bolt11,
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
    let node = LightningRPC::new(RPC_FILE);
    let invoices = node
        .listinvoices(Some(&payment_id))
        .expect("fetched invoices");
    let invoice = invoices.invoices[0].clone();

    let payment_complete = invoice.status == "paid";

    let response = PaymentStatusResponse {
        payment_complete: payment_complete,
        // For later doing onchain
        confirmed_paid: 0,
        unconfirmed_paid: 0,
    };
    (Status::Accepted, Json(response))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        // .attach(Main::init())
        .mount("/", FileServer::from("./html"))
        .mount("/api", routes![create_payment, check_payment])
}
