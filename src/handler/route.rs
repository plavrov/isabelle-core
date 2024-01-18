use crate::handler::intranet::*;
use crate::handler::equestrian::*;
use crate::handler::security::*;
use crate::state::store::Store;
use crate::State;
use actix_identity::Identity;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use log::info;
use std::collections::HashMap;

pub async fn call_item_pre_edit_hook(
    srv: &mut crate::state::data::Data,
    hndl: &str,
    user: &Option<Item>,
    collection: &str,
    old_itm: Option<Item>,
    itm: &mut Item,
    del: bool,
    merge: bool,
) -> ProcessResult {
    match hndl {
        "security_password_challenge_pre_edit_hook" => {
            return security_password_challenge_pre_edit_hook(
                srv, user, collection, old_itm, itm, del, merge,
            )
            .await;
        }
        "security_check_unique_login_email" => {
            return security_check_unique_login_email(
                srv, user, collection, old_itm, itm, del, merge,
            )
            .await;
        }
        &_ => {
            return ProcessResult {
                succeeded: true,
                error: "".to_string(),
            };
        }
    }
}

pub async fn call_item_post_edit_hook(
    srv: &mut crate::state::data::Data,
    hndl: &str,
    collection: &str,
    id: u64,
    del: bool,
) {
    match hndl {
        "equestrian_job_sync" => equestrian_job_sync(srv, collection, id, del).await,
        &_ => {}
    }
}

pub async fn call_itm_auth_hook(
    srv: &mut crate::state::data::Data,
    hndl: &str,
    user: &Option<Item>,
    collection: &str,
    id: u64,
    new_item: Option<Item>,
    del: bool,
) -> bool {
    match hndl {
        "equestrian_itm_auth_hook" => {
            return equestrian_itm_auth_hook(srv, user, collection, id, new_item, del).await;
        }
        "intranet_itm_auth_hook" => {
            return intranet_itm_auth_hook(srv, user, collection, id, new_item, del).await;
        }
        &_ => return false,
    }
}

pub async fn call_itm_list_filter_hook(
    mut srv: &mut crate::state::data::Data,
    hndl: &str,
    user: &Option<Item>,
    collection: &str,
    context: &str,
    map: &mut HashMap<u64, Item>,
) {
    match hndl {
        "equestrian_itm_filter_hook" => {
            return equestrian_itm_filter_hook(&mut srv, user, collection, context, map).await;
        }
        "security_itm_filter_hook" => {
            return security_itm_filter_hook(&mut srv, user, collection, context, map).await;
        }
        &_ => {}
    }
}

pub async fn call_url_route(
    mut srv: &mut crate::state::data::Data,
    user: Identity,
    hndl: &str,
    query: &str,
) -> HttpResponse {
    match hndl {
        "equestrian_schedule_materialize" => {
            return equestrian_schedule_materialize(&mut srv, user, query).await;
        }
        "equestrian_pay_find_broken_payments" => {
            return equestrian_pay_find_broken_payments(&mut srv, user, query).await;
        }
        "equestrian_pay_deactivate_expired_payments" => {
            return equestrian_pay_deactivate_expired_payments(&mut srv, user, query).await;
        }
        "equestrian_event_subscribe" => {
            return equestrian_event_subscribe(&mut srv, user, query).await;
        }
        &_ => {
            return HttpResponse::NotFound().into();
        }
    }
}

pub async fn url_route(
    user: Identity,
    data: actix_web::web::Data<State>,
    req: HttpRequest,
) -> HttpResponse {
    let mut srv = data.server.lock().unwrap();
    let routes = srv
        .rw
        .get_internals()
        .await
        .safe_strstr("extra_route", &HashMap::new());

    info!("Custom URL: {}", req.path());

    for route in routes {
        let parts: Vec<&str> = route.1.split(":").collect();
        if parts[0] == req.path() {
            info!("Call custom route {}", parts[2]);
            return call_url_route(&mut srv, user, parts[2], req.query_string()).await;
        }
    }

    HttpResponse::NotFound().into()
}

pub async fn call_collection_read_hook(hndl: &str, collection: &str, itm: &mut Item) -> bool {
    match hndl {
        "security_collection_read_hook" => {
            return security_collection_read_hook(collection, itm).await;
        }
        _ => {
            return false;
        }
    }
}

pub async fn call_otp_hook(srv: &mut crate::state::data::Data, hndl: &str, itm: Item) {
    match hndl {
        "security_otp_send_email" => {
            security_otp_send_email(srv, itm).await;
        }
        _ => {}
    }
}
