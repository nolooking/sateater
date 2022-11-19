use configparser::ini::Ini;
use lettre::Message;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};

// uses ~/.aws/credentials
pub async fn send_email(subject: String, body: String) {
    let mut config = Ini::new();
    let _map = config
        .load("./config.cfg")
        .expect("config.cfg does not exist! please copy config_example.cfg");
    let emailsender = config
        .get("email", "sender")
        .expect("email sender specified in config");

    let email = Message::builder()
        .from(
            format!("nolooking Forward <{}>", emailsender)
                .parse()
                .unwrap(),
        )
        .to(format!("nolooking Forward <{}>", emailsender)
            .parse()
            .unwrap())
        .subject(subject.clone())
        .body(body.clone())
        .unwrap();

    let raw_email = email.formatted();
    let ses_client = SesClient::new(rusoto_core::Region::UsEast1);
    let ses_request = SendRawEmailRequest {
        raw_message: RawMessage {
            data: base64::encode(raw_email).into(),
        },
        ..Default::default()
    };

    if let Err(e) = ses_client.send_raw_email(ses_request).await {
        eprintln!("FAILED TO SEND EMAIL:\n\n{}\n\n{}\n\n{}", subject, body, e);
    };
}
