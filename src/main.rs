
mod data_model;
mod server;
use actix_identity::Identity;
use actix_web::{get, post, web, App, HttpResponse, HttpRequest, HttpServer, Responder, cookie::Key, cookie::SameSite};
use actix_web::web::Data;
use crate::server::state::*;
use serde::{Deserialize, Serialize};

use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_identity::IdentityMiddleware;
use actix_cors::Cors;
use log::{info, error};
use crate::data_model::user::*;
use crate::data_model::mentee::*;
use crate::data_model::schedule_entry::*;
use crate::server::data_reader::*;
use std::ops::DerefMut;
use std::ops::Deref;

async fn user_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpUser {
        #[serde(default = "unset_id")]
        pub id: u64,
        pub firstname: String,
        pub surname: String,
    }

    let mut c = User::new();
    let params = web::Query::<TmpUser>::from_query(req.query_string()).unwrap();

    let mut srv = data.server.lock().unwrap();

    let mut idx = srv.users_cnt + 1;
    if params.id == unset_id() {
        srv.users_cnt += 1;
    }
    else {
        idx = params.id;
        srv.users.remove(&idx);
    }

    c.firstname = params.firstname.clone();
    c.surname = params.surname.clone();

    if params.id == unset_id() {
        info!("Added user {} {} with ID {}", &params.firstname.to_string(), &params.surname.to_string(), idx);
    }
    else {
        info!("Edited user {} {} with ID {}", &params.firstname.to_string(), &params.surname.to_string(), idx);
    }
    srv.users.insert(idx, c);

    HttpResponse::Ok()
}

async fn user_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {

    #[derive(Deserialize, Debug)]
    pub struct UserDelParams {
        id: u64,
    }

    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<UserDelParams>::from_query(req.query_string()).unwrap();
    if srv.users.contains_key(&params.id) {
        srv.users.remove(&params.id);
        info!("Removed user with ID {}", &params.id);
    } else {
        error!("Failed to remove user {}", params.id);
    }

    HttpResponse::Ok()
}

async fn user_list(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpUser {
        pub id: u64,
        pub firstname: String,
        pub surname: String,
    }

    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpUserList {
        pub users: Vec<TmpUser>
    }

    let _srv = data.server.lock().unwrap();

    let mut lst = TmpUserList {
        users: Vec::new(),
    };

    for el in &_srv.users {
        lst.users.push(TmpUser {
            id: *el.0 as u64,
            firstname: el.1.firstname.clone(),
            surname: el.1.surname.clone(),
        });
    }

    web::Json(lst)
}


async fn mentee_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpParams {
        #[serde(default = "unset_id")]
        pub id: u64,
        pub name: String,
    }

    let mut c = Mentee::new();
    let params = web::Query::<TmpParams>::from_query(req.query_string()).unwrap();

    let mut srv = data.server.lock().unwrap();

    let mut idx = srv.mentee_cnt + 1;
    if params.id == unset_id() {
        srv.mentee_cnt += 1;
    }
    else {
        idx = params.id;
        srv.mentees.remove(&idx);
    }

    c.name = params.name.clone();

    if params.id == unset_id() {
        info!("Added mentee {} with ID {}", &params.name.to_string(), srv.users_cnt);
    }
    else {
        info!("Edited mentee {} with ID {}", &params.name.to_string(), srv.users_cnt);
    }
    srv.mentees.insert(idx, c);

    HttpResponse::Ok()
}

async fn mentee_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {

    #[derive(Deserialize, Debug)]
    pub struct TmpParams {
        id: u64,
    }

    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<TmpParams>::from_query(req.query_string()).unwrap();
    if srv.mentees.contains_key(&params.id) {
        srv.mentees.remove(&params.id);
        info!("Removed mentees with ID {}", &params.id);
    } else {
        error!("Failed to remove mentee {}", params.id);
    }

    HttpResponse::Ok()
}

async fn mentee_list(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpMentee {
        pub id: u64,
        pub name: String,
    }

    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpMenteeList {
        pub mentees: Vec<TmpMentee>
    }

    let _srv = data.server.lock().unwrap();

    let mut lst = TmpMenteeList {
         mentees: Vec::new(),
    };

    for el in &_srv.mentees {
        lst.mentees.push(TmpMentee {
            id: *el.0 as u64,
            name: el.1.name.clone(),
        });
    }

    web::Json(lst)
}

fn unset_id() -> u64 {
    return u64::MAX;
}

async fn schedule_entry_edit(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpParams {
        #[serde(default = "unset_id")]
        pub id: u64,
        pub is_group: bool,
        pub mentees: Vec<u64>,
        pub users: Vec<u64>,
        pub time: u64,
    }

    let mut c = ScheduleEntry::new();
    let params = web::Query::<TmpParams>::from_query(req.query_string()).unwrap();

    let mut srv = data.server.lock().unwrap();

    let mut idx = srv.schedule_entry_cnt + 1;
    if params.id != unset_id() {
        srv.schedule_entry_cnt += 1;
    }
    else {
        idx = params.id;
    }

    c.is_group = params.is_group;
    c.mentees = params.mentees.clone();
    c.users = params.users.clone();
    c.times.push(params.time);

    if params.id != unset_id() {
        if srv.schedule_entries.contains_key(&params.id) {
            for time in srv.schedule_entries[&params.id].times.clone()
            {
                if srv.schedule_entry_times.contains_key(&time)
                {
                    srv.schedule_entry_times.get_mut(&time).unwrap().retain(|&val| val != params.id);
                }
            }
            srv.schedule_entries.remove(&params.id);
        }
    }

    srv.schedule_entries.insert(idx, c);
    if params.id == unset_id()
    {
        info!("Added new schedule entry with ID {}", idx);
    }
    else
    {
        info!("Edited schedule entry with ID {}", idx);
    }


    let mut obj = srv.schedule_entry_times[&params.time].clone();
    obj.push(idx);
    *srv.schedule_entry_times.get_mut(&params.time).unwrap() = obj;

    HttpResponse::Ok()
}

async fn schedule_entry_del(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {

    #[derive(Deserialize, Debug)]
    pub struct TmpParams {
        id: u64,
    }

    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<TmpParams>::from_query(req.query_string()).unwrap();
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

    HttpResponse::Ok()
}


async fn schedule_entry_list(_user: Option<Identity>, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpParams {
        #[serde(default = "unset_id")]
        pub after: u64,
        #[serde(default = "unset_id")]
        pub before: u64,
    }

    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpScheduleEntry {
        pub timestamp: u64,
        pub is_group: bool,
        pub mentees: Vec<u64>,
        pub users: Vec<u64>,
        pub times: Vec<u64>,
    }

    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpScheduleEntryList {
        pub schedule_entries: Vec<TmpScheduleEntry>
    }

    let params = web::Query::<TmpParams>::from_query(req.query_string()).unwrap();
    let _srv = data.server.lock().unwrap();

    let mut lst = TmpScheduleEntryList {
         schedule_entries: Vec::new(),
    };

    for el in &_srv.schedule_entries {
        for time in &el.1.times {
            if params.after != unset_id() && params.after > *time {
                continue;
            }
            if params.before != unset_id() && params.before < *time {
                continue;
            }
        }
        let mut entry = TmpScheduleEntry {
            timestamp: *el.0 as u64,
            is_group: el.1.is_group,
            mentees: Vec::new(),
            users: Vec::new(),
            times: Vec::new()
        };
        for mentee in &el.1.mentees {
            entry.mentees.push(*mentee);
        }
        for user in &el.1.users {
            entry.users.push(*user);
        }
        for time in &el.1.times {
            entry.times.push(*time);
        }
        lst.schedule_entries.push(entry);
    }

    web::Json(lst)
}

// The secret key would usually be read from a configuration file/environment variables.
fn get_secret_key() -> Key {
    return Key::generate();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    //std::env::set_var("RUST_LOG", "debug");
    let state = State::new();
    {
        let mut srv = state.server.lock().unwrap();
        srv.users_cnt = 5;
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
