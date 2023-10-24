use isabelle_dm::data_model::item::Item;
use std::ops::Deref;

use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_qs;
use serde_qs::Config;

use crate::state::state::*;

use log::info;

use crate::notif::gcal::*;
use crate::server::user_control::*;
use crate::state::data_rw::*;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

pub async fn setting_edit(
    user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    let config = Config::new(10, false);
    let c: Item = config.deserialize_str(&_req.query_string()).unwrap();
    srv.settings = c.clone();
    info!("Setting edit: {}", serde_json::to_string(&c.strs).unwrap());
    write_data(srv.deref_mut());
    HttpResponse::Ok().into()
}

pub async fn setting_list(
    user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    let st = srv.settings.clone();
    HttpResponse::Ok().body(serde_json::to_string(&st).unwrap()).into()
}

pub async fn setting_gcal_auth(
    user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    HttpResponse::Ok().body(auth_google(&srv)).into()
}

pub async fn setting_gcal_auth_end(
    user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    info!("Auth end");
    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct AuthEndData {
        pub state: String,
        pub code: String,
        pub scope: String,
    }

    let config = Config::new(10, false);
    let data: AuthEndData = config.deserialize_str(&_req.query_string()).unwrap();

    HttpResponse::Ok().body(auth_google_end(
        &srv,
        srv.public_url.clone() + "/?" + _req.query_string(),
        data.state,
        data.code,
    )).into()
}
