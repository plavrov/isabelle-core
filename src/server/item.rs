use isabelle_dm::util::accessor::unset_id;

use isabelle_dm::data_model::del_param::DelParam;
use isabelle_dm::data_model::item::Item;
use std::ops::Deref;

use serde_qs;

use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::state::state::*;

use log::{error, info};

use crate::state::data_rw::*;
use std::ops::DerefMut;

use crate::server::user_control::*;

pub async fn item_edit(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    let mut c = serde_qs::from_str::<Item>(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.items_cnt + 1;

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None
        || (!current_user
            .as_ref()
            .unwrap()
            .bool_params
            .contains_key("role_is_admin")
            && !current_user
                .as_ref()
                .unwrap()
                .bool_params
                .contains_key("role_is_teacher"))
    {
        info!("Item edit: no user");
        return HttpResponse::Unauthorized();
    }

    if c.id == unset_id() {
        srv.items_cnt += 1;
    } else {
        idx = c.id;
        srv.items.remove(&idx);
    }

    c.id = idx;

    if c.id == unset_id() {
        info!(
            "Added item {} {} with ID {}",
            &c.safe_str("firstname", "".to_string()).to_string(),
            &c.safe_str("surname", "".to_string()).to_string(),
            idx
        );
    } else {
        info!(
            "Edited item {} {} with ID {}",
            &c.safe_str("firstname", "".to_string()).to_string(),
            &c.safe_str("surname", "".to_string()).to_string(),
            idx
        );
    }
    srv.items.insert(idx, c);

    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

pub async fn item_del(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None
        || (!current_user
            .as_ref()
            .unwrap()
            .bool_params
            .contains_key("role_is_admin")
            && !current_user
                .as_ref()
                .unwrap()
                .bool_params
                .contains_key("role_is_teacher"))
    {
        info!("Item del: no user");
        return HttpResponse::Unauthorized();
    }

    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();
    if srv.items.contains_key(&params.id) {
        srv.items.remove(&params.id);
        info!("Removed item with ID {}", &params.id);
    } else {
        error!("Failed to remove item {}", params.id);
    }

    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

pub async fn item_done(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    let mut srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None
        || (!current_user
            .as_ref()
            .unwrap()
            .bool_params
            .contains_key("role_is_admin")
            && !current_user
                .as_ref()
                .unwrap()
                .bool_params
                .contains_key("role_is_teacher"))
    {
        info!("Item done: no user");
        return HttpResponse::Unauthorized();
    }

    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();
    if srv.items.contains_key(&params.id) {
        let mut itm = srv.items[&params.id].clone();

        srv.items.remove(&params.id);
        if itm.bool_params.contains_key("done") {
            let obj = itm.bool_params.get_mut("done").unwrap();
            *obj = true;
        } else {
            itm.bool_params.insert("done".to_string(), true);
        }
        srv.items.insert(params.id, itm);
        info!("Marked item with ID {} as done", &params.id);
    } else {
        error!("Failed to mark item {} as done", params.id);
    }

    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

pub async fn item_list(_user: Identity, data: web::Data<State>) -> HttpResponse {
    let srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None
        || (!current_user
            .as_ref()
            .unwrap()
            .bool_params
            .contains_key("role_is_admin")
            && !current_user
                .as_ref()
                .unwrap()
                .bool_params
                .contains_key("role_is_teacher"))
    {
        info!("Item list: no user");
        return HttpResponse::Unauthorized().into();
    }

    HttpResponse::Ok().body(serde_json::to_string(&srv.items).unwrap())
}
