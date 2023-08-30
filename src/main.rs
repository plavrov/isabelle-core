mod server;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::del_param::DelParam;
use isabelle_dm::data_model::schedule_entry::ScheduleEntry;
use serde_qs;
use serde_qs::Config;
use actix_identity::Identity;
use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder, cookie::Key, cookie::SameSite};
use actix_web::web::Data;
use crate::server::state::*;

use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_identity::IdentityMiddleware;
use actix_cors::Cors;
use log::{info, error};

use crate::server::data_rw::*;
use std::ops::DerefMut;

async fn item_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut c = serde_qs::from_str::<Item>(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.items_cnt + 1;

    if c.id == unset_id() {
        srv.items_cnt += 1;
    }
    else {
        idx = c.id;
        srv.items.remove(&idx);
    }

    c.id = idx;

    if c.id == unset_id() {
        info!("Added item {} {} with ID {}", &c.safe_str("firstname", "".to_string()).to_string(), &c.safe_str("surname", "".to_string()).to_string(), idx);
    }
    else {
        info!("Edited item {} {} with ID {}", &c.safe_str("firstname", "".to_string()).to_string(), &c.safe_str("surname", "".to_string()).to_string(), idx);
    }
    srv.items.insert(idx, c);

    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn item_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();
    if srv.items.contains_key(&params.id) {
        srv.items.remove(&params.id);
        info!("Removed item with ID {}", &params.id);
    } else {
        error!("Failed to remove item {}", params.id);
    }

    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn item_list(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    let _srv = data.server.lock().unwrap();

    web::Json(_srv.items.clone())
}

fn unset_id() -> u64 {
    return u64::MAX;
}

async fn schedule_entry_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let mut c : ScheduleEntry = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.schedule_entry_cnt + 1;

    info!("Entry: {}", serde_json::to_string(&c.clone()).unwrap());

    if c.id == unset_id() {
        srv.schedule_entry_cnt += 1;
    }
    else {
        idx = c.id;
    }

    if c.id != unset_id() {
        if srv.schedule_entries.contains_key(&c.id) {
            let time = c.time;
            if srv.schedule_entry_times.contains_key(&time)
            {
                srv.schedule_entry_times.get_mut(&time).unwrap().retain(|&val| val != c.id);
            }
            info!("Removed old schedule entry with ID {}", idx);
            srv.schedule_entries.remove(&c.id);
        }
    }

    c.id = idx;
    if c.id == unset_id()
    {
        info!("Added new schedule entry with ID {}", idx);
    }
    else
    {
        info!("Edited schedule entry with ID {}", idx);
    }

    let time = c.time;
    if !srv.schedule_entry_times.contains_key(&time) {
        srv.schedule_entry_times.insert(time, Vec::new());
    }


    let mut obj = srv.schedule_entry_times[&time].clone();
    obj.push(idx);
    *srv.schedule_entry_times.get_mut(&time).unwrap() = obj;

    srv.schedule_entries.insert(idx, c);
    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn schedule_entry_done(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let mut c : ScheduleEntry = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();

    let mut nc = srv.schedule_entries[&c.id].clone();

    if nc.bool_params.contains_key("done") {
        let obj = nc.bool_params.get_mut("done").unwrap();
        *obj = true;
    }
    else {
        nc.bool_params.insert("done".to_string(), true);
    }

    srv.schedule_entries.remove(&c.id);
    srv.schedule_entries.insert(c.id, nc);

    if c.id != unset_id()
    {
        info!("Marked schedule entry with ID {} as done", c.id);
    }

    //write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn schedule_entry_paid(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let mut c : ScheduleEntry = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();

    let mut nc = srv.schedule_entries[&c.id].clone();

    if nc.bool_params.contains_key("paid") {
        let obj = nc.bool_params.get_mut("paid").unwrap();
        *obj = true;
    }
    else {
        nc.bool_params.insert("paid".to_string(), true);
    }

    srv.schedule_entries.remove(&c.id);
    srv.schedule_entries.insert(c.id, nc);

    if c.id != unset_id()
    {
        info!("Marked schedule entry with ID {} as paid", c.id);
    }

    //write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}


async fn schedule_entry_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();

    if srv.schedule_entries.contains_key(&params.id) {

        let time = srv.schedule_entries[&params.id].time;
        {
            if srv.schedule_entry_times.contains_key(&time)
            {
                srv.schedule_entry_times.get_mut(&time).unwrap().retain(|&val| val != params.id);
            }
        }
        srv.schedule_entries.remove(&params.id);
        info!("Removed schedule entry with ID {}", &params.id);
    } else {
        error!("Failed to remove schedule entry {}", params.id);
    }

    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}


async fn schedule_entry_list(_user: Option<Identity>, data: web::Data<State>, _req: HttpRequest) -> impl Responder {
    let _srv = data.server.lock().unwrap();
    web::Json(_srv.schedule_entries.clone())
}

// The secret key would usually be read from a configuration file/environment variables.
fn get_secret_key() -> Key {
    return Key::generate();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let state = State::new();
    {
        let mut srv = state.server.lock().unwrap();
        {
            *srv.deref_mut() = read_data("sample-data")
        }
    }
    let data = Data::new(state);
    let secret_key = get_secret_key();
    info!("Starting server");
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Cors::permissive())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_key.clone()
                )
                .cookie_same_site(SameSite::None)
                .cookie_secure(false)
                .cookie_http_only(false)
                .build(),
            )
            .route("/item/edit", web::get().to(item_edit))
            .route("/item/del", web::get().to(item_del))
            .route("/item/list", web::get().to(item_list))
            .route("/schedule/edit", web::get().to(schedule_entry_edit))
            .route("/schedule/del", web::get().to(schedule_entry_del))
            .route("/schedule/list", web::get().to(schedule_entry_list))
            .route("/schedule/done", web::get().to(schedule_entry_done))
            .route("/schedule/paid", web::get().to(schedule_entry_paid))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
