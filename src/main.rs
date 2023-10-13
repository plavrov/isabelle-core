use chrono::NaiveDateTime;
use chrono::DateTime;
use std::collections::HashMap;
use std::ops::Deref;
mod notif;
mod server;
use actix_session::config::{BrowserSession, CookieContentSecurity};
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::del_param::DelParam;
use isabelle_dm::data_model::schedule_entry::ScheduleEntry;
use isabelle_dm::data_model::all_settings::AllSettings;
use serde_qs;
use serde_qs::Config;
use actix_identity::Identity;
use actix_web::{web, App, HttpMessage, HttpResponse, HttpRequest, HttpServer, Responder, cookie::Key, cookie::SameSite};
use actix_web::web::Data;
use crate::server::state::*;

use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_identity::IdentityMiddleware;
use actix_cors::Cors;

use log::{info, error};
use std::env;
use crate::server::data_rw::*;
use std::ops::DerefMut;
use serde::{Deserialize, Serialize};
use chrono::{Utc};
use now::DateTimeNow;
use crate::notif::gcal::*;
use crate::notif::email::*;

fn get_user(srv: &crate::server::data::Data, login: String) -> Option<Item> {
    for item in &srv.items {
        if item.1.fields.contains_key("login") &&
           item.1.fields["login"] == login &&
           item.1.bool_params.contains_key("is_human") {
            return Some(item.1.clone());
        }
    }
    return None;
}

async fn item_edit(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut c = serde_qs::from_str::<Item>(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.items_cnt + 1;

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Item edit: no user");
        return HttpResponse::Unauthorized();
    }

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

    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

async fn item_del(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
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

async fn item_list(_user: Identity, data: web::Data<State>) -> HttpResponse {
    let srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Item list: no user");
        return HttpResponse::Unauthorized().into();
    }


    HttpResponse::Ok().body(serde_json::to_string(&srv.items).unwrap())
}

fn unset_id() -> u64 {
    return u64::MAX;
}

fn unset_week() -> u64 {
    return 0;
}

pub fn eventname(srv: &crate::server::data::Data, sch: &ScheduleEntry) -> String {
    let teacher_id = sch.safe_id("teacher", 0);
    if teacher_id == 0 {
        "Training".to_string()
    } else {
        "Training with ".to_owned() + &srv.items[&teacher_id].safe_str("firstname", "<unknown>".to_string())
    }
}

pub fn entry2datetimestr(entry: &ScheduleEntry) -> String {
    #![allow(warnings)]
    let mut datetime = entry.time;

    let all_days = [ "mon", "tue", "wed", "thu", "fri", "sat", "sun" ];
    let day = entry.safe_str("day_of_the_week", "".to_string());
    if day != "" && day != "unset" {
        let now = Utc::now();
        let tmp_day = all_days.iter().position(|&r| r == day).unwrap() as u64;
        datetime = (now.beginning_of_week().timestamp() as u64) + 24 * 60 * 60 * tmp_day + (entry.time % (24 * 60 * 60));
    }

    if datetime == 0 {
        datetime = chrono::Local::now().timestamp() as u64;
    }

    let naive = NaiveDateTime::from_timestamp(datetime as i64, 0);
    let utc_date_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let newdate = utc_date_time.format("%Y-%m-%d %H:%M");
    newdate.to_string()
}

async fn schedule_entry_edit(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let mut c : ScheduleEntry = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.schedule_entry_cnt + 1;

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Schedule entry edit: no user");
        return HttpResponse::Unauthorized();
    }

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
            init_google(&srv);
            sync_with_google(&srv,
                    false,
                    eventname(&srv, &srv.schedule_entries[&c.id]),
                    entry2datetimestr(&srv.schedule_entries[&c.id]));
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


    /* emails */
    let entities: [&str; 2] = ["teacher", "student"];
    let email_entities: [&str; 2] = ["email", "parent_email"];

    // Part 2: loop over elements in string array.
    for ent in &entities {
        for em in &email_entities {
            let target_id = c.safe_id(ent, 0);
            if srv.items.contains_key(&target_id) {
                let target = &srv.items[&target_id];
                let target_email = target.safe_str(em, "".to_string());
                if target.safe_bool("notify_training_email", false) &&
                   target_email != "" {
                    send_email(&srv,
                        &target_email,
                        "Schedule changed",
                        &format!("Please review changes for the following entry:\n{}{}",
                        srv.public_url.clone() + "/job/edit?id=",
                        &idx.to_string()));
                }
            }
        }
    }

    {
        let target_id = c.safe_id("student", 0);
        if srv.items.contains_key(&target_id) {
            let target = &srv.items[&target_id];
            let target_email = target.safe_str("email", "".to_string());
            if target.safe_bool("notify_training_email", false) &&
               target_email != "" {
                send_email(&srv,
                    &target_email,
                    "Schedule changed",
                    &format!("Please review changes for the following entry:\n{}{}",
                    srv.public_url.clone() + "/job/edit?id=",
                    &idx.to_string()));
            }
        }
    }

    let mut obj = srv.schedule_entry_times[&time].clone();
    obj.push(idx);
    *srv.schedule_entry_times.get_mut(&time).unwrap() = obj;

    init_google(&srv);
    sync_with_google(&srv,
                    true,
                    eventname(&srv, &c),
                    entry2datetimestr(&c));
    srv.schedule_entries.insert(idx, c);
    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

async fn schedule_entry_done(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let c : ScheduleEntry = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Schedule entry done: no user");
        return HttpResponse::Unauthorized();
    }

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

    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

async fn schedule_entry_paid(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let c : ScheduleEntry = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Schedule entry paid: no user");
        return HttpResponse::Unauthorized();
    }

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

    write_data(srv.deref_mut());

    HttpResponse::Ok()
}


async fn schedule_entry_del(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<DelParam>::from_query(req.query_string()).unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Schedule entry del: no user");
        return HttpResponse::Unauthorized();
    }

    init_google(&srv);

    if srv.schedule_entries.contains_key(&params.id) {

        let time = srv.schedule_entries[&params.id].time;
        {
            if srv.schedule_entry_times.contains_key(&time)
            {
                srv.schedule_entry_times.get_mut(&time).unwrap().retain(|&val| val != params.id);
            }
        }
        let ent = &srv.schedule_entries[&params.id];
        sync_with_google(&srv, false, eventname(&srv, &ent), entry2datetimestr(ent));
        srv.schedule_entries.remove(&params.id);
        info!("Removed schedule entry with ID {}", &params.id);
    } else {
        error!("Failed to remove schedule entry {}", params.id);
    }

    write_data(srv.deref_mut());
    HttpResponse::Ok()
}


async fn schedule_entry_list(_user: Identity, data: web::Data<State>, _req: HttpRequest) -> HttpResponse {
    let srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       (!current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") &&
        !current_user.as_ref().unwrap().bool_params.contains_key("role_is_teacher")) {
        info!("Item list: no user");
        return HttpResponse::Unauthorized().into();
    }

    HttpResponse::Ok().body(serde_json::to_string(&srv.schedule_entries).unwrap())
}

async fn schedule_materialize(_user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    info!("Query: {}", &req.query_string());

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    struct WeekSchedule {
        #[serde(default = "unset_week")]
        pub week : u64,
    }

    let params = web::Query::<WeekSchedule>::from_query(req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut vec : Vec<ScheduleEntry> = Vec::new();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None ||
       !current_user.as_ref().unwrap().bool_params.contains_key("role_is_admin") {
        info!("Schedule entry paid: no user");
        return HttpResponse::Unauthorized();
    }

    info!("WEEK: {}", params.week);

    let now = Utc::now();
    let week_start = (now.beginning_of_week().timestamp() as u64) + (60 * 60 * 24 * 7) * params.week;
    let mut final_cnt = srv.schedule_entry_cnt;
    for entry in &srv.schedule_entries {
        let day = entry.1.safe_str("day_of_the_week", "".to_string());
        let pid = entry.1.safe_id("parent_id", u64::MAX);
        if day != "" && day != "unset" && pid == u64::MAX {
            let mut cp_entry = ScheduleEntry::new();
            info!("Found entry that we want to materialize: {}", entry.0);
            let all_days = [ "mon", "tue", "wed", "thu", "fri", "sat", "sun" ];
            let tmp_day = all_days.iter().position(|&r| r == day).unwrap() as u64;
            let ts = week_start + (60 * 60 * 24) * tmp_day + entry.1.time % (60 * 60 * 24);
            cp_entry.time = ts;
            cp_entry.id_params.insert("parent_id".to_string(), *entry.0);
            cp_entry.str_params.insert("day_of_the_week".to_string(), "unset".to_string());

            let mut skip = false;
            for tmp__ in &srv.schedule_entries {
                if tmp__.1.time == cp_entry.time &&
                   tmp__.1.safe_id("parent_id", u64::MAX) == *entry.0 {
                    skip = true;
                    break;
                }
            }

            if !skip {
                final_cnt += 1;
                cp_entry.id = final_cnt;
                vec.push(cp_entry);
            }
        }
    }

    for ent in vec {
        info!("Materialized entry with ID {}", ent.id);
        srv.schedule_entries.insert(ent.id, ent);
    }

    srv.schedule_entry_cnt = final_cnt;

    write_data(srv.deref_mut());

    HttpResponse::Ok()
}

async fn login(_user: Option<Identity>, _data: web::Data<State>, request: HttpRequest) -> impl Responder {
    let srv = _data.server.lock().unwrap();

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct LoginUser {
        pub username: String,
        pub password: String,
    }

    let config = Config::new(10, false);
    let c : LoginUser = config.deserialize_str(&request.query_string()).unwrap();
    let mut found : bool = false;

    for item in &srv.items {
        if item.1.bool_params.contains_key("is_human") {
            info!("{} / {} against {} / {}",
                  item.1.fields["login"], item.1.fields["password"],
                  c.username, c.password);
        }
        if item.1.bool_params.contains_key("is_human") &&
           item.1.fields.contains_key("login") &&
           item.1.fields["login"] == c.username &&
           item.1.fields["password"] == c.password {
            Identity::login(&request.extensions(), c.username.clone()).unwrap();
            info!("Logged in! {}", c.username);
            found = true;
            break;
        }
    }

    if !found {
        info!("No user found, couldn't log in");
    }

    HttpResponse::Ok()
}

async fn logout(_user: Identity, _data: web::Data<State>, _request: HttpRequest) -> impl Responder {
    _user.logout();
    info!("Logged out");

    HttpResponse::Ok()
}

async fn is_logged_in(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    let srv = data.server.lock().unwrap();

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct LoginUser {
        pub username: String,
        pub id: u64,
        pub role: Vec<String>,
    }

    let mut user : LoginUser = LoginUser { username: "".to_string(), id: 0, role: Vec::new() };

    if _user.is_none() {
        info!("No user");
        return web::Json(user)
    }

    for item in &srv.items {
        if item.1.fields.contains_key("login") &&
           item.1.fields["login"] == _user.as_ref().unwrap().id().unwrap() {
            if item.1.bool_params.contains_key("is_human") {
                user.username = _user.as_ref().unwrap().id().unwrap();
                user.id = *item.0;
                for bp in &item.1.bool_params {
                    if bp.0.starts_with("role_is_") {
                        user.role.push(bp.0[8..].to_string());
                    }
                }
                break;
            }
        }
    }
    web::Json(user)
}

async fn setting_edit(_user: Identity, data: web::Data<State>, _req: HttpRequest) -> impl Responder {
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

async fn setting_list(_user: Identity, data: web::Data<State>, _req: HttpRequest) -> HttpResponse {
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

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(
        CookieSessionStore::default(), Key::from(&[0; 64])
    )
    .session_lifecycle(BrowserSession::default())
    .cookie_same_site(SameSite::None)
    .cookie_path("/".into())
    .cookie_name(String::from("isabelle-cookie"))
    .cookie_domain(Some("localhost".into()))
    .cookie_content_security(CookieContentSecurity::Private)
    .cookie_http_only(true)
    .cookie_secure(true)
    .build()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut gc_path : String = "".to_string();
    let mut py_path : String = "".to_string();
    let mut data_path : String = "sample-data".to_string();
    let mut pub_path : String = "http://localhost:8081".to_string();
    let mut gc_next = false;
    let mut py_next = false;
    let mut data_next = false;
    let mut pub_next = false;
    for arg in args {
        if gc_next {
            gc_path = arg.clone();
            gc_next = false;
        } else if py_next {
            py_path = arg.clone();
            py_next = false;
        } else if data_next {
            data_path = arg.clone();
            data_next = false;
        } else if pub_next {
            pub_path = arg.clone();
            pub_next = false;
        }

        if arg == "--gc-path" {
            gc_next = true;
        } else if arg == "--py-path" {
            py_next = true;
        } else if arg == "--data-path" {
            data_next = true;
        } else if arg == "--pub-url" {
            pub_next = true;
        }
    }

    env_logger::init();

    let state = State::new();
    {
        let mut srv = state.server.lock().unwrap();
        {
            *srv.deref_mut() = read_data(&data_path);
            (*srv.deref_mut()).gc_path = gc_path.to_string();
            (*srv.deref_mut()).py_path = py_path.to_string();
            (*srv.deref_mut()).data_path = data_path.to_string();
            (*srv.deref_mut()).public_url = pub_path.to_string();

            init_google(srv.deref_mut());
        }
    }

    let data = Data::new(state);
    info!("Starting server");
    HttpServer::new(move ||
        App::new()
            .app_data(data.clone())
            .wrap(Cors::permissive())
            .wrap(IdentityMiddleware::default())
            .wrap(session_middleware())
            .route("/item/edit", web::post().to(item_edit))
            .route("/item/del", web::post().to(item_del))
            .route("/item/list", web::get().to(item_list))
            .route("/schedule/edit", web::post().to(schedule_entry_edit))
            .route("/schedule/del", web::post().to(schedule_entry_del))
            .route("/schedule/list", web::get().to(schedule_entry_list))
            .route("/schedule/done", web::post().to(schedule_entry_done))
            .route("/schedule/paid", web::post().to(schedule_entry_paid))
            .route("/schedule/materialize", web::post().to(schedule_materialize))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/is_logged_in", web::get().to(is_logged_in))
            .route("/setting/edit", web::post().to(setting_edit))
            .route("/setting/list", web::get().to(setting_list))
    )
    .bind(("127.0.0.1", 8090))?
    .run()
    .await
}
