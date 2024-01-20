use crate::state::store::Store;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use log::{error, info};

pub async fn send_email(srv: &mut crate::state::data::Data, to: &str, subject: &str, body: &str) {
    info!("Checking options...");

    let settings = srv.rw.get_settings().await.clone();

    let smtp_server = settings.safe_str("smtp_server", "");
    let smtp_login = settings.safe_str("smtp_login", "");
    let smtp_password = settings.safe_str("smtp_password", "");
    let smtp_from = settings.safe_str("smtp_from", "");

    info!("Building email...");

    if to == "" || smtp_server == "" || smtp_login == "" || smtp_password == "" || smtp_from == "" {
        info!("Input options not present");
        return;
    }

    let email = Message::builder()
        .from(smtp_from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(String::from(body))
        .unwrap();

    let creds = Credentials::new(smtp_login.to_owned(), smtp_password.to_owned());

    info!("Sending email...");
    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => error!("Could not send email: {:?}", e),
    }
}
