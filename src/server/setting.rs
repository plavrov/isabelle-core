use isabelle_dm::data_model::item::Item;
use std::ops::Deref;

use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
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
    _user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> impl Responder {
    let mut _srv = data.server.lock().unwrap();

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None
        || (!current_user
            .as_ref()
            .unwrap()
            .bools
            .contains_key("role_is_admin")
            && !current_user
                .as_ref()
                .unwrap()
                .bools
                .contains_key("role_is_teacher"))
    {
        info!("Setting edit: no user");
        return HttpResponse::Unauthorized();
    }

    let config = Config::new(10, false);
    let c: Item = config.deserialize_str(&_req.query_string()).unwrap();
    _srv.settings = c.clone();
    info!(
        "Setting edit: {}",
        serde_json::to_string(&c.strs).unwrap()
    );
    write_data(_srv.deref_mut());
    HttpResponse::Ok()
}

pub async fn setting_list(
    _user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let _srv = data.server.lock().unwrap();

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None
        || (!current_user
            .as_ref()
            .unwrap()
            .bools
            .contains_key("role_is_admin")
            && !current_user
                .as_ref()
                .unwrap()
                .bools
                .contains_key("role_is_teacher"))
    {
        info!("Setting list: no user");
        return HttpResponse::Unauthorized().finish();
    }

    let st = _srv.settings.clone();
    HttpResponse::Ok().body(serde_json::to_string(&st).unwrap())
}

pub async fn setting_gcal_auth(
    _user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let _srv = data.server.lock().unwrap();

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None
        || !current_user
            .as_ref()
            .unwrap()
            .bools
            .contains_key("role_is_admin")
    {
        info!("Setting list: no user");
        return HttpResponse::Unauthorized().finish();
    }

    HttpResponse::Ok().body(auth_google(&_srv))
}

pub async fn setting_gcal_auth_end(
    _user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let _srv = data.server.lock().unwrap();

    info!("Auth end");
    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct AuthEndData {
        pub state: String,
        pub code: String,
        pub scope: String,
    }

    let config = Config::new(10, false);
    let data: AuthEndData = config.deserialize_str(&_req.query_string()).unwrap();

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None
        || !current_user
            .as_ref()
            .unwrap()
            .bools
            .contains_key("role_is_admin")
    {
        info!("Setting list: no user");
        return HttpResponse::Unauthorized().finish();
    }

    HttpResponse::Ok().body(auth_google_end(
        &_srv,
        _srv.public_url.clone() + "/?" + _req.query_string(),
        data.state,
        data.code,
    ))
}
