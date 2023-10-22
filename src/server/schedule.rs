use chrono::DateTime;
use chrono::NaiveDateTime;
use isabelle_dm::util::accessor::unset_id;

use std::ops::Deref;

use isabelle_dm::data_model::id_param::IdParam;
use isabelle_dm::data_model::item::Item;

use crate::notif::email::*;
use crate::notif::gcal::*;
use crate::state::data_rw::*;
use crate::state::state::*;
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use log::{error, info};
use now::DateTimeNow;
use serde::{Deserialize, Serialize};
use serde_qs;
use serde_qs::Config;
use std::ops::DerefMut;

use crate::server::user_control::*;

pub fn eventname(srv: &crate::state::data::Data, sch: &Item) -> String {
    let teacher_id = sch.safe_id("teacher", 0);
    if teacher_id == 0 {
        "Training".to_string()
    } else {
        "Training with ".to_owned()
            + &srv.items[&teacher_id].safe_str("firstname", "<unknown>")
    }
}

pub fn entry2datetimestr(entry: &Item) -> String {
    #![allow(warnings)]
    let mut datetime = entry.u64s["time"];

    let all_days = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    let day = entry.safe_str("day_of_the_week", "");
    if day != "" && day != "unset" {
        let now = Utc::now();
        let tmp_day = all_days.iter().position(|&r| r == day).unwrap() as u64;
        datetime = (now.beginning_of_week().timestamp() as u64)
            + 24 * 60 * 60 * tmp_day
            + (entry.u64s["time"] % (24 * 60 * 60));
    }

    if datetime == 0 {
        datetime = chrono::Local::now().timestamp() as u64;
    }

    let naive = NaiveDateTime::from_timestamp(datetime as i64, 0);
    let utc_date_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let newdate = utc_date_time.format("%Y-%m-%d %H:%M");
    newdate.to_string()
}

pub async fn schedule_entry_edit(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let mut c: Item = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut idx = srv.schedule_entry_cnt + 1;

    let current_user = get_user(srv.deref(), _user.id().unwrap());
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
        info!("Schedule entry edit: no user");
        return HttpResponse::Unauthorized();
    }

    info!("Entry: {}", serde_json::to_string(&c.clone()).unwrap());

    if c.id == unset_id() {
        srv.schedule_entry_cnt += 1;
    } else {
        idx = c.id;
    }

    if c.id != unset_id() {
        if srv.schedule_entries.contains_key(&c.id) {
            let time = c.safe_u64("time", 0);
            if srv.schedule_entry_times.contains_key(&time) {
                srv.schedule_entry_times
                    .get_mut(&time)
                    .unwrap()
                    .retain(|&val| val != c.id);
            }
            info!("Removed old schedule entry with ID {}", idx);
            init_google(&srv);
            sync_with_google(
                &srv,
                false,
                eventname(&srv, &srv.schedule_entries[&c.id]),
                entry2datetimestr(&srv.schedule_entries[&c.id]),
            );
            srv.schedule_entries.remove(&c.id);
        }
    }

    c.id = idx;
    if c.id == unset_id() {
        info!("Added new schedule entry with ID {}", idx);
    } else {
        info!("Edited schedule entry with ID {}", idx);
    }

    let time = c.safe_u64("time", 0);
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
                let target_email = target.safe_str(em, "");
                if target.safe_bool("notify_training_email", false) && target_email != "" {
                    send_email(
                        &srv,
                        &target_email,
                        "Schedule changed",
                        &format!(
                            "Please review changes for the following entry:\n{}{}",
                            srv.public_url.clone() + "/job/edit?id=",
                            &idx.to_string()
                        ),
                    );
                }
            }
        }
    }

    {
        let target_id = c.safe_id("student", 0);
        if srv.items.contains_key(&target_id) {
            let target = &srv.items[&target_id];
            let target_email = target.safe_str("email", "");
            if target.safe_bool("notify_training_email", false) && target_email != "" {
                send_email(
                    &srv,
                    &target_email,
                    "Schedule changed",
                    &format!(
                        "Please review changes for the following entry:\n{}{}",
                        srv.public_url.clone() + "/job/edit?id=",
                        &idx.to_string()
                    ),
                );
            }
        }
    }

    let mut obj = srv.schedule_entry_times[&time].clone();
    obj.push(idx);
    *srv.schedule_entry_times.get_mut(&time).unwrap() = obj;

    init_google(&srv);
    sync_with_google(&srv, true, eventname(&srv, &c), entry2datetimestr(&c));
    srv.schedule_entries.insert(idx, c);
    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

pub async fn schedule_entry_done(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let c: Item = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
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
        info!("Schedule entry done: no user");
        return HttpResponse::Unauthorized();
    }

    let mut nc = srv.schedule_entries[&c.id].clone();

    if nc.bools.contains_key("done") {
        let obj = nc.bools.get_mut("done").unwrap();
        *obj = true;
    } else {
        nc.bools.insert("done".to_string(), true);
    }

    srv.schedule_entries.remove(&c.id);
    srv.schedule_entries.insert(c.id, nc);

    if c.id != unset_id() {
        info!("Marked schedule entry with ID {} as done", c.id);
    }

    write_data(srv.deref_mut());
    HttpResponse::Ok()
}

pub async fn schedule_entry_paid(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    info!("Query: {}", &req.query_string());
    let config = Config::new(10, false);
    let c: Item = config.deserialize_str(&req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
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
        info!("Schedule entry paid: no user");
        return HttpResponse::Unauthorized();
    }

    let mut nc = srv.schedule_entries[&c.id].clone();

    if nc.bools.contains_key("paid") {
        let obj = nc.bools.get_mut("paid").unwrap();
        *obj = true;
    } else {
        nc.bools.insert("paid".to_string(), true);
    }

    srv.schedule_entries.remove(&c.id);
    srv.schedule_entries.insert(c.id, nc);

    if c.id != unset_id() {
        info!("Marked schedule entry with ID {} as paid", c.id);
    }

    write_data(srv.deref_mut());

    HttpResponse::Ok()
}

pub async fn schedule_entry_del(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let params = web::Query::<IdParam>::from_query(req.query_string()).unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
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
        info!("Schedule entry del: no user");
        return HttpResponse::Unauthorized();
    }

    init_google(&srv);

    if srv.schedule_entries.contains_key(&params.id) {
        let time = srv.schedule_entries[&params.id].u64s["time"];
        {
            if srv.schedule_entry_times.contains_key(&time) {
                srv.schedule_entry_times
                    .get_mut(&time)
                    .unwrap()
                    .retain(|&val| val != params.id);
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

pub async fn schedule_entry_list(
    _user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
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
        info!("Item list: no user");
        return HttpResponse::Unauthorized().into();
    }

    HttpResponse::Ok().body(serde_json::to_string(&srv.schedule_entries).unwrap())
}

fn unset_week() -> u64 {
    return 0;
}

pub async fn schedule_materialize(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    info!("Query: {}", &req.query_string());

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    struct WeekSchedule {
        #[serde(default = "unset_week")]
        pub week: u64,
    }

    let params = web::Query::<WeekSchedule>::from_query(req.query_string()).unwrap();
    let mut srv = data.server.lock().unwrap();
    let mut vec: Vec<Item> = Vec::new();

    let current_user = get_user(srv.deref(), _user.id().unwrap());
    if current_user == None
        || !current_user
            .as_ref()
            .unwrap()
            .bools
            .contains_key("role_is_admin")
    {
        info!("Schedule entry paid: no user");
        return HttpResponse::Unauthorized();
    }

    info!("WEEK: {}", params.week);

    let now = Utc::now();
    let week_start =
        (now.beginning_of_week().timestamp() as u64) + (60 * 60 * 24 * 7) * params.week;
    let mut final_cnt = srv.schedule_entry_cnt;
    for entry in &srv.schedule_entries {
        let day = entry.1.safe_str("day_of_the_week", "");
        let pid = entry.1.safe_id("parent_id", u64::MAX);
        if day != "" && day != "unset" && pid == u64::MAX {
            let mut cp_entry = Item::new();
            info!("Found entry that we want to materialize: {}", entry.0);
            let all_days = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
            let tmp_day = all_days.iter().position(|&r| r == day).unwrap() as u64;
            let ts = week_start + (60 * 60 * 24) * tmp_day + entry.1.u64s["time"] % (60 * 60 * 24);
            cp_entry.set_u64("time", ts);
            cp_entry.ids.insert("parent_id".to_string(), *entry.0);
            cp_entry
                .strs
                .insert("day_of_the_week".to_string(), "unset".to_string());

            let mut skip = false;
            for tmp__ in &srv.schedule_entries {
                if tmp__.1.u64s["time"] == cp_entry.u64s["time"]
                    && tmp__.1.safe_id("parent_id", u64::MAX) == *entry.0
                {
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
