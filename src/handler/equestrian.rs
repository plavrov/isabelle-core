use std::collections::HashMap;
use crate::state::data_rw::*;
use chrono::DateTime;
use chrono::NaiveDateTime;
use isabelle_dm::data_model::item::Item;
use std::ops::Deref;

use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use std::ops::DerefMut;

use crate::server::user_control::*;

use crate::init_google;
use crate::notif::email::send_email;
use crate::notif::gcal::sync_with_google;
use log::info;
use now::DateTimeNow;

pub fn eventname(srv: &crate::state::data::Data, sch: &Item) -> String {
    let teacher_id = sch.safe_id("teacher", 0);
    if teacher_id == 0 {
        "Training".to_string()
    } else {
        "Training with ".to_owned()
            + &srv.itm["user"]
                .get(teacher_id)
                .unwrap()
                .safe_str("firstname", "<unknown>")
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

pub fn equestrian_job_sync(
    srv: &mut crate::state::data::Data,
    collection: &str,
    id: u64,
    del: bool,
) {
    if collection != "job" {
        info!("Equestrian job sync: not job");
        return;
    }

    let j = srv.itm["job"].get(id);
    if j == None {
        info!("Equestrian job sync: no job");
        return;
    }

    info!("Equestrian job sync: starting");
    let job = j.unwrap();

    /* emails */
    let entities: [&str; 2] = ["teacher", "student"];
    let email_entities: [&str; 2] = ["email", "parent_email"];

    // Part 2: loop over elements in string array.
    for ent in &entities {
        for em in &email_entities {
            let user_id = job.safe_id(ent, 0);
            let user = srv.itm["user"].get(user_id);
            if user != None {
                info!(
                    "Found user: {}",
                    user.as_ref().unwrap().safe_str("firstname", "")
                );
                let target_email = user.as_ref().unwrap().safe_str(em, "");
                if user
                    .as_ref()
                    .unwrap()
                    .safe_bool("notify_training_email", false)
                    && target_email != ""
                {
                    info!("Target email found");
                    if del {
                        send_email(
                            &srv,
                            &target_email,
                            "Schedule changed",
                            "The schedule entry has been removed. Please review your new schedule",
                        );
                    } else {
                        send_email(
                            &srv,
                            &target_email,
                            "Schedule changed",
                            &format!(
                                "Please review changes for the following entry:\n{}{}",
                                srv.public_url.clone() + "/job/edit?id=",
                                &id.to_string()
                            ),
                        );
                    }
                } else {
                    info!("Target email not found");
                }
            }
        }
    }

    init_google(&srv);
    sync_with_google(
        &srv,
        if del { false } else { true },
        eventname(&srv, &job),
        entry2datetimestr(&job),
    );
}

fn unset_week() -> u64 {
    return 0;
}

pub fn equestrian_schedule_materialize(
    mut srv: &mut crate::state::data::Data,
    user: Identity,
    query: &str,
) -> HttpResponse {
    info!("Query: {}", query);

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    struct WeekSchedule {
        #[serde(default = "unset_week")]
        pub week: u64,
    }

    let params = web::Query::<WeekSchedule>::from_query(query).unwrap();
    let mut vec: Vec<Item> = Vec::new();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, &usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    info!("WEEK: {}", params.week);

    let now = Utc::now();
    let week_start =
        (now.beginning_of_week().timestamp() as u64) + (60 * 60 * 24 * 7) * params.week;
    let mut final_cnt = srv.itm["job"].count;
    for entry in srv.itm["job"].get_all() {
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
            for tmp__ in srv.itm["job"].get_all() {
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
        srv.itm.get_mut("job").unwrap().set(ent.id, ent, false);
    }

    write_data(srv.deref_mut());
    HttpResponse::Ok().into()
}

pub fn equestrian_pay_find_broken_payments(
    srv: &mut crate::state::data::Data,
    user: Identity,
    query: &str,
) -> HttpResponse {
    let usr = get_user(&srv, user.id().unwrap());

    if check_role(&srv, &usr, "admin") {
        return HttpResponse::Unauthorized().into();
    }

    info!("Query: {}", query);

    HttpResponse::Ok().into()
}

pub fn equestrian_itm_auth_hook(
    srv: &mut crate::state::data::Data,
    user: &Option<Item>,
    collection: &str,
    id: u64,
    _del: bool) -> bool {
    if check_role(&srv, &user, "admin") {
        return true;
    }

    info!("Checking collection {} user id {}", collection, user.as_ref().unwrap().id);

    if collection == "query" &&
       (check_role(&srv, &user, "student") ||
        check_role(&srv, &user, "teacher") ||
        check_role(&srv, &user, "staff")) {
        let itm = srv.itm["query"].get(id);
        if !itm.is_none() &&
           itm.unwrap().safe_id("requester", u64::MAX) == user.as_ref().unwrap().id {
            return true;
        }
        return false;
    } else if collection == "job" &&
              (check_role(&srv, &user, "teacher") ||
               check_role(&srv, &user, "staff")) {
        return true;
    } else if collection == "mentee" &&
              (check_role(&srv, &user, "teacher") ||
               check_role(&srv, &user, "staff")) {
        return true;
    } else if collection == "user" {
        let itm = srv.itm["user"].get(id);
        if !itm.is_none() && itm.unwrap().id == user.as_ref().unwrap().id {
            return true;
        }
        return false;
    }

    return false;
}

pub fn equestrian_itm_filter_hook(
    srv: &crate::state::data::Data,
    user: &Option<Item>,
    collection: &str,
    map: &mut HashMap<u64, Item>) {
    if check_role(&srv, &user, "admin") {
        return;
    }

    info!("Checking collection {} user id {}", collection, user.as_ref().unwrap().id);

    if collection == "user" {
        for el in map {
            el.1.strs.remove("password");
        }
    }
}
