use sateater::load_conf;
use tonic_lnd::{lnrpc::AddInvoiceResponse, LndLightningClient};

use futures_util::StreamExt;

use crate::inbound::{build_and_send_email, load_inbound_requests};

pub async fn get_lnd_client() -> LndLightningClient {
    let (address, cert, macaroon, _) = load_conf();
    tonic_lnd::connect_lightning(address, cert, macaroon)
        .await
        .expect("failed to connect")
}

pub async fn get_onchain_address() -> String {
    let mut client = get_lnd_client().await;

    let newaddressreq = tonic_lnd::lnrpc::NewAddressRequest {
        r#type: 4, //taproot
        ..Default::default()
    };

    let address = client
        .new_address(newaddressreq)
        .await
        .unwrap()
        .into_inner();
    address.address
}

pub async fn get_info() -> tonic_lnd::lnrpc::GetInfoResponse {
    let mut client = get_lnd_client().await;

    let inforeq = tonic_lnd::lnrpc::GetInfoRequest {};
    let info = client.get_info(inforeq).await.unwrap().into_inner();
    info
}

pub async fn create_invoice(amount: i64, description: String) -> AddInvoiceResponse {
    let mut client = get_lnd_client().await;

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
        .unwrap()
        .into_inner();

    created_invoice
}

pub async fn check_invoice(payment_id: String) -> bool {
    let mut client = get_lnd_client().await;

    let payment_hash = tonic_lnd::lnrpc::PaymentHash {
        r_hash: hex::decode(payment_id).expect("valid payment hash"),
        ..Default::default()
    };

    let invoice = client
        .lookup_invoice(payment_hash)
        .await
        .expect("fetched invoices")
        .into_inner();

    invoice.state == 1
}

#[tokio::main]
pub async fn monitor_onchain_received() {
    let mut client = get_lnd_client().await;

    let tx_req = tonic_lnd::lnrpc::GetTransactionsRequest {
        end_height: get_info().await.block_height as i32,
        ..Default::default()
    };

    let mut stream = client
        .subscribe_transactions(tx_req)
        .await
        .expect("fetched stream")
        .into_inner();

    let mut seen_txs = vec![];
    while let Some(tx) = stream.next().await {
        let inbound_requests = load_inbound_requests().await;
        let tx = tx.expect("valid tx");

        // Hack to avoid duplicates( where are they coming from? )
        if seen_txs.contains(&tx.tx_hash) {
            continue;
        }
        seen_txs.push(tx.tx_hash);

        let outputs = tx.output_details;
        for output in outputs {
            if let Some(request) = inbound_requests
                .iter()
                .find(|req| req.payment_address == output.address)
            {
                if request.price <= output.amount as u64 {
                    build_and_send_email(request).await;
                    println!("We received a payment! Sent email!");
                    println!("{:?}\n", request);
                } else {
                    println!("Looks like someone underpaid {:?}", request);
                }
            }
        }
    }
}
