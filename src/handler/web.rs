use futures_util::TryStreamExt;
use isabelle_dm::data_model::item::Item;
use actix_multipart::Multipart;
use crate::state::store::Store;
use crate::notif::email::send_email;
use actix_web::HttpResponse;

use actix_identity::Identity;

use log::info;

pub async fn web_contact(
    mut srv: &mut crate::state::data::Data,
    _user: Option<Identity>,
    query: &str,
    mut payload: Multipart,
) -> HttpResponse {
    info!("Contact from the website");
    let mut itm = serde_qs::from_str::<Item>(query).unwrap();

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Ok(Some(chunk)) = field.try_next().await {
            let data = chunk;

            if field.name() == "item" {
                let v = &data.to_vec();
                let strv = std::str::from_utf8(v).unwrap_or("{}");
                let new_itm: Item = serde_json::from_str(strv).unwrap_or(Item::new());
                itm.merge(&new_itm);
            }
        }
    }

    let from_email = itm.safe_str("from_email", "");
    let from_name = itm.safe_str("from_name", "");
    let topic = itm.safe_str("topic", "");
    let message = itm.safe_str("message", "");

    let to_email = srv
        .rw
        .get_settings()
        .await
        .clone()
        .safe_str("web_contact_email", "");
    send_email(
        &mut srv,
        &to_email,
        &format!("Website: {} ({})", topic, from_name),
        &format!("From: {}\nName: {}\nTopic: {}\n\n{}", from_email, from_name, topic, message),
    )
    .await;
    HttpResponse::Ok().into()
}
