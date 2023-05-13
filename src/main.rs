mod server;
use isabelle_dm::data_model::user::User;
use isabelle_dm::data_model::mentee::Mentee;
use isabelle_dm::data_model::del_param::DelParam;
use isabelle_dm::data_model::schedule_entry::ScheduleEntry;
use serde_qs;
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

async fn user_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut c = serde_qs::from_str::<User>(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.users_cnt + 1;

    if c.id == unset_id() {
        srv.users_cnt += 1;
    }
    else {
        idx = c.id;
        srv.users.remove(&idx);
    }

    c.id = idx;

    if c.id == unset_id() {
        info!("Added user {} {} with ID {}", &c.firstname.to_string(), &c.surname.to_string(), idx);
    }
    else {
        info!("Edited user {} {} with ID {}", &c.firstname.to_string(), &c.surname.to_string(), idx);
    }
    srv.users.insert(idx, c);

    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn user_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();
    if srv.users.contains_key(&params.id) {
        srv.users.remove(&params.id);
        info!("Removed user with ID {}", &params.id);
    } else {
        error!("Failed to remove user {}", params.id);
    }

    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn user_list(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    let _srv = data.server.lock().unwrap();

    web::Json(_srv.users.clone())
}


async fn mentee_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut c = serde_qs::from_str::<Mentee>(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.mentee_cnt + 1;

    if c.id == unset_id() {
        srv.mentee_cnt += 1;
    }
    else {
        idx = c.id;
        srv.mentees.remove(&idx);
    }

    if c.id == unset_id() {
        info!("Added mentee {} with ID {}", &c.name.to_string(), srv.users_cnt);
    }
    else {
        info!("Edited mentee {} with ID {}", &c.name.to_string(), srv.users_cnt);
    }
    c.id = idx;
    srv.mentees.insert(idx, c);

    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn mentee_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();

    if srv.mentees.contains_key(&params.id) {
        srv.mentees.remove(&params.id);
        info!("Removed mentees with ID {}", &params.id);
    } else {
        error!("Failed to remove mentee {}", params.id);
    }

    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn mentee_list(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    let _srv = data.server.lock().unwrap();

    web::Json(_srv.mentees.clone())
}

fn unset_id() -> u64 {
    return u64::MAX;
}

async fn schedule_entry_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut c = serde_qs::from_str::<ScheduleEntry>(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.schedule_entry_cnt + 1;

    if c.id == unset_id() {
        srv.schedule_entry_cnt += 1;
    }
    else {
        idx = c.id;
    }

    if c.id != unset_id() {
        if srv.schedule_entries.contains_key(&c.id) {
            for time in srv.schedule_entries[&c.id].times.clone()
            {
                if srv.schedule_entry_times.contains_key(&time)
                {
                    srv.schedule_entry_times.get_mut(&time).unwrap().retain(|&val| val != c.id);
                }
            }
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

    for time in &c.times {
        if !srv.schedule_entry_times.contains_key(&time) {
            srv.schedule_entry_times.insert(*time, Vec::new());
        }


        let mut obj = srv.schedule_entry_times[&time].clone();
        obj.push(idx);
        *srv.schedule_entry_times.get_mut(&time).unwrap() = obj;
    }
    srv.schedule_entries.insert(idx, c);
    write_data(srv.deref_mut(), "sample-data");
    HttpResponse::Ok()
}

async fn schedule_entry_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();

    if srv.schedule_entries.contains_key(&params.id) {

        for time in srv.schedule_entries[&params.id].times.clone()
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
            .route("/user/edit", web::get().to(user_edit))
            .route("/user/del", web::get().to(user_del))
            .route("/user/list", web::get().to(user_list))
            .route("/mentee/edit", web::get().to(mentee_edit))
            .route("/mentee/del", web::get().to(mentee_del))
            .route("/mentee/list", web::get().to(mentee_list))
            .route("/schedule/edit", web::get().to(schedule_entry_edit))
            .route("/schedule/del", web::get().to(schedule_entry_del))
            .route("/schedule/list", web::get().to(schedule_entry_list))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
