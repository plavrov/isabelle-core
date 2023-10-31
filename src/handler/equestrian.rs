use crate::state::data_rw::*;
use chrono::DateTime;
use chrono::NaiveDateTime;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use std::collections::HashMap;
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

pub fn date2ts(date: String, time: String) -> u64 {
    #![allow(warnings)]
    let ndt = NaiveDateTime::parse_from_str(
        &(date.to_string() + " " + &time.to_string()),
        "%Y-%m-%d %H:%M",
    );
    return ndt.unwrap().timestamp() as u64;
}

pub fn eventname(srv: &crate::state::data::Data, sch: &Item) -> String {
    let teacher_id = sch.safe_id("teacher", 0);
    if teacher_id == 0 {
        "Training".to_string()
    } else {
        "Training with ".to_owned()
            + &srv.itm["user"]
                .get(teacher_id)
                .unwrap()
                .safe_str("name", "<unknown>")
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
                    user.as_ref().unwrap().safe_str("name", "")
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
                vec.push(cp_entry);
            }
        }
    }

    for ent in vec {
        info!("Materialized entry with ID {}", ent.id);
        srv.itm.get_mut("job").unwrap().set(ent.id, ent, false);
    }

    write_data(srv.deref_mut());
    HttpResponse::Ok().body(
        serde_json::to_string(&ProcessResult {
            succeeded: true,
            error: "".to_string(),
        })
        .unwrap(),
    )
}

pub fn equestrian_pay_find_broken_payments(
    srv: &mut crate::state::data::Data,
    user: Identity,
    query: &str,
) -> HttpResponse {
    let usr = get_user(&srv, user.id().unwrap());

    if !check_role(&srv, &usr, "admin") {
        return HttpResponse::Unauthorized().into();
    }

    info!("Query: {}", query);

    HttpResponse::Ok().into()
}

pub fn equestrian_pay_deactivate_expired_payments(
    srv: &mut crate::state::data::Data,
    user: Identity,
    _query: &str,
) -> HttpResponse {
    let usr = get_user(&srv, user.id().unwrap());
    let now_time = chrono::Local::now().timestamp() as u64;

    if !check_role(&srv, &usr, "admin") {
        return HttpResponse::Unauthorized().into();
    }

    info!("Deactivate expired payments");

    let mut updated_payments: Vec<Item> = Vec::new();
    let jobs = srv.itm["job"].get_all();
    for pay in srv.itm["payment"].get_all() {
        let id = pay.0;
        let mut new_pay = pay.1.clone();
        let mut use_new = false;

        if pay.1.safe_str("payment_type", "") == "monthly" {
            let time: u64;
            let months = [
                "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
            ];
            let mon_str = pay.1.safe_str("target_month", "jan");
            let year_str = pay.1.safe_str("target_year", "0");
            let mut mon = months.iter().position(|&x| x == mon_str).unwrap() + 1 + 1;
            let mut year = year_str.parse::<u64>().unwrap();
            if mon == 13 {
                mon = 1;
                year += 1;
            }

            time = date2ts(
                year.to_string() + "-" + &mon.to_string() + "-01",
                "00:00".to_string(),
            );
            info!(
                "Payment ID {}: time {} {} / {} {} = {} vs now {}",
                pay.0,
                mon_str,
                year_str,
                mon.to_string(),
                year.to_string(),
                time,
                now_time
            );
            if time < now_time {
                info!("Expire payment with ID {}", pay.0);
                new_pay.set_bool("inactive", true);
                use_new = true;
            }
        }

        let assoc_jobs: Vec<_> = jobs
            .iter()
            .filter(|x| &x.1.safe_id("payment_id", u64::MAX) == id)
            .collect();
        let no_lessons = new_pay.safe_u64("no_lessons", 0);
        let real_used_lessons = assoc_jobs.len() as u64;
        if pay.1.safe_u64("used_lessons", 0) != real_used_lessons {
            info!(
                "Break payment with ID {}: {}",
                pay.0,
                real_used_lessons > no_lessons
            );
            new_pay.set_u64("used_lessons", real_used_lessons);
            new_pay.set_bool("broken", real_used_lessons > no_lessons);
            use_new = true;
        }

        if use_new {
            updated_payments.push(new_pay);
        }
    }

    for pay in updated_payments {
        srv.itm.get_mut("payment").unwrap().set(pay.id, pay, false);
    }

    HttpResponse::Ok().into()
}

pub fn equestrian_itm_auth_hook(
    srv: &mut crate::state::data::Data,
    user: &Option<Item>,
    collection: &str,
    id: u64,
    new_item: Option<Item>,
    _del: bool,
) -> bool {
    if check_role(&srv, &user, "admin") {
        return true;
    }

    info!(
        "Checking collection {} user id {}",
        collection,
        user.as_ref().unwrap().id
    );

    if collection == "query"
        && (check_role(&srv, &user, "student")
            || check_role(&srv, &user, "teacher")
            || check_role(&srv, &user, "staff"))
    {
        let mut accept = true;
        let itm = srv.itm["query"].get(id);

        if !itm.is_none()
            && itm.unwrap().safe_id("requester", u64::MAX) != user.as_ref().unwrap().id
        {
            accept = false;
        }

        if !new_item.is_none()
            && new_item.unwrap().safe_id("requester", u64::MAX) != user.as_ref().unwrap().id
        {
            accept = false;
        }

        return accept;
    } else if collection == "job"
        && (check_role(&srv, &user, "teacher") || check_role(&srv, &user, "staff"))
    {
        return true;
    } else if collection == "mentee"
        && (check_role(&srv, &user, "teacher") || check_role(&srv, &user, "staff"))
    {
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
    context: &str,
    map: &mut HashMap<u64, Item>,
) {
    let mut list = true;

    if check_role(&srv, &user, "admin") && collection != "user" {
        return;
    }

    if context == "full" {
        list = false;
    }

    let mut short_map: HashMap<u64, Item> = HashMap::new();
    if user.is_none() {
        *map = short_map;
        return;
    }

    info!(
        "Checking collection {} user id {}",
        collection,
        user.as_ref().unwrap().id
    );
    if list {
        for el in &mut *map {
            if collection == "user" {
                let mut itm = Item::new();
                itm.id = *el.0;
                itm.strs
                    .insert("name".to_string(), el.1.safe_str("name", ""));
                itm.bools.insert(
                    "role_is_teacher".to_string(),
                    el.1.safe_bool("role_is_teacher", false),
                );
                itm.bools.insert(
                    "role_is_student".to_string(),
                    el.1.safe_bool("role_is_student", false),
                );
                itm.bools.insert(
                    "role_is_staff".to_string(),
                    el.1.safe_bool("role_is_staff", false),
                );
                short_map.insert(*el.0, itm);
            } else if collection == "payment" {
                let mut itm = Item::new();
                itm.id = *el.0;
                itm.ids
                    .insert("requester".to_string(), el.1.safe_id("requester", u64::MAX));
                itm.strs.insert(
                    "payment_type".to_string(),
                    el.1.safe_str("payment_type", ""),
                );
                itm.strs.insert(
                    "target_month".to_string(),
                    el.1.safe_str("target_month", ""),
                );
                itm.strs
                    .insert("target_year".to_string(), el.1.safe_str("target_year", ""));
                itm.u64s
                    .insert("no_lessons".to_string(), el.1.safe_u64("no_lessons", 0));
                short_map.insert(*el.0, itm);
            } else {
                let mut itm = Item::new();
                itm.id = *el.0;
                itm.strs
                    .insert("name".to_string(), el.1.safe_str("name", ""));
                short_map.insert(*el.0, itm);
            }
        }
    } else {
        if collection == "user" {
            for el in &mut *map {
                if *el.0 != user.as_ref().unwrap().id {
                    let mut itm = Item::new();
                    itm.id = *el.0;
                    itm.strs
                        .insert("name".to_string(), el.1.safe_str("name", ""));
                    itm.bools.insert(
                        "role_is_teacher".to_string(),
                        el.1.safe_bool("role_is_teacher", false),
                    );
                    itm.bools.insert(
                        "role_is_student".to_string(),
                        el.1.safe_bool("role_is_student", false),
                    );
                    itm.bools.insert(
                        "role_is_staff".to_string(),
                        el.1.safe_bool("role_is_staff", false),
                    );
                    short_map.insert(*el.0, itm);
                } else {
                    let mut itm = el.1.clone();
                    if itm.strs.contains_key("salt") {
                        itm.strs.remove("salt");
                    }
                    if itm.strs.contains_key("password") {
                        itm.strs.remove("password");
                    }
                    short_map.insert(*el.0, itm);
                }
            }
        } else if collection == "job" {
            for el in &mut *map {
                if el.1.safe_id("student", u64::MAX) == user.as_ref().unwrap().id {
                    short_map.insert(*el.0, el.1.clone());
                }
            }
        } else if collection == "query" {
            for el in &mut *map {
                if el.1.safe_id("requester", u64::MAX) == user.as_ref().unwrap().id {
                    short_map.insert(*el.0, el.1.clone());
                }
            }
        } else {
            short_map = map.clone();
        }
    }
    *map = short_map;
}
