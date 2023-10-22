

use std::collections::HashMap;
use std::ops::Deref;





use isabelle_dm::data_model::all_settings::AllSettings;
use serde_qs;
use serde_qs::Config;
use actix_identity::Identity;
use actix_web::{web, HttpResponse, HttpRequest, Responder};

use crate::state::state::*;






use log::{info};

use crate::state::data_rw::*;
use std::ops::DerefMut;
use serde::{Deserialize, Serialize};
use crate::notif::gcal::*;
use crate::server::user_control::*;

pub async fn setting_edit(_user: Identity, data: web::Data<State>, _req: HttpRequest) -> impl Responder {
    let mut _srv = data.server.lock().unwrap();

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Setting edit: no user");
        return HttpResponse::Unauthorized();
    }

    let config = Config::new(10, false);
    let c : AllSettings = config.deserialize_str(&_req.query_string()).unwrap();
    _srv.settings = c.clone();
    info!("Setting edit: {}", serde_json::to_string(&c.str_params).unwrap());
    write_data(_srv.deref_mut());
    HttpResponse::Ok()
}

pub async fn setting_list(_user: Identity, data: web::Data<State>, _req: HttpRequest) -> HttpResponse {
    let _srv = data.server.lock().unwrap();
    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct AllSettings {
        pub str_params: HashMap<String, String>,
    }

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Setting list: no user");
        return HttpResponse::Unauthorized().finish();
    }

    let st = _srv.settings.clone();
    HttpResponse::Ok().body(serde_json::to_string(&st).unwrap())
}

pub async fn setting_gcal_auth(_user: Identity, data: web::Data<State>, _req: HttpRequest) -> HttpResponse {
    let _srv = data.server.lock().unwrap();

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None ||
       !current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") {
        info!("Setting list: no user");
        return HttpResponse::Unauthorized().finish();
    }

    HttpResponse::Ok().body(auth_google(&_srv))
}

pub async fn setting_gcal_auth_end(_user: Identity, data: web::Data<State>, _req: HttpRequest) -> HttpResponse {
    let _srv = data.server.lock().unwrap();

    info!("Auth end");
    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct AuthEndData {
        pub state: String,
        pub code: String,
        pub scope: String,
    }

    let config = Config::new(10, false);
    let data : AuthEndData = config.deserialize_str(&_req.query_string()).unwrap();

    let current_user = get_user(_srv.deref(), _user.id().unwrap());
    if current_user == None ||
       !current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") {
        info!("Setting list: no user");
        return HttpResponse::Unauthorized().finish();
    }

    HttpResponse::Ok().body(auth_google_end(&_srv, _srv.public_url.clone() + "/?" + _req.query_string(), data.state, data.code))
}
