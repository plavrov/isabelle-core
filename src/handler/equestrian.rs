use crate::state::store::Store;
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

pub async fn eventname(srv: &mut crate::state::data::Data, sch: &Item) -> String {
    let mut teacher_id = sch.safe_id("teacher", u64::MAX);
    if teacher_id == 0 {
        teacher_id = u64::MAX;
    }

    let itm = srv.rw.get_item("user", teacher_id).await;
    if teacher_id == u64::MAX || itm.is_none() {
        "Training".to_string()
    } else {
        "Training with ".to_owned() + &itm.unwrap().safe_str("name", "<unknown>")
    }
}

pub fn entry2datetimestr(entry: &Item) -> String {
    #![allow(warnings)]
    let mut datetime = entry.safe_u64("time", 0);

    let all_days = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    let day = entry.safe_str("day_of_the_week", "");
    if day != "" && day != "unset" {
        let now = Utc::now();
        let tmp_day = all_days.iter().position(|&r| r == day).unwrap() as u64;
        datetime = (now.beginning_of_week().timestamp() as u64)
            + 24 * 60 * 60 * tmp_day
            + (entry.safe_u64("time", 0) % (24 * 60 * 60));
    }

    if datetime == 0 {
        datetime = chrono::Local::now().timestamp() as u64;
    }

    let naive = NaiveDateTime::from_timestamp(datetime as i64, 0);
    let utc_date_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let newdate = utc_date_time.format("%Y-%m-%d %H:%M");
    newdate.to_string()
}

pub async fn equestrian_job_sync(
    mut srv: &mut crate::state::data::Data,
    collection: &str,
    id: u64,
    del: bool,
) {
    if collection != "job" {
        info!("Equestrian job sync: not job");
        return;
    }

    let j = srv.rw.get_item("job", id).await;
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
            let user = srv.rw.get_item("user", user_id).await;
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
                            &mut srv,
                            &target_email,
                            "Schedule changed",
                            "The schedule entry has been removed. Please review your new schedule",
                        )
                        .await;
                    } else {
                        let public_url = srv.public_url.clone();
                        send_email(
                            &mut srv,
                            &target_email,
                            "Schedule changed",
                            &format!(
                                "Please review changes for the following entry:\n{}{}",
                                public_url + "/job/edit?id=",
                                &id.to_string()
                            ),
                        )
                        .await;
                    }
                } else {
                    info!("Target email not found");
                }
            }
        }
    }

    init_google(srv).await;
    let event_name = eventname(&mut srv, &job).await;
    sync_with_google(&mut srv, !del, event_name, entry2datetimestr(&job)).await;
}

fn unset_week() -> u64 {
    return 0;
}

fn unset_id() -> u64 {
    return u64::MAX;
}

pub async fn equestrian_schedule_materialize(
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
    let usr = get_user(&mut srv, user.id().unwrap()).await;

    if !check_role(&mut srv, &usr, "admin").await {
        return HttpResponse::Forbidden().into();
    }

    info!("WEEK: {}", params.week);

    let now = Utc::now();
    let week_start =
        (now.beginning_of_week().timestamp() as u64) + (60 * 60 * 24 * 7) * params.week;
    let all_jobs = srv.rw.get_all_items("job", "id", "").await;
    for entry in &all_jobs.map {
        let day = entry.1.safe_str("day_of_the_week", "");
        let pid = entry.1.safe_id("parent_id", u64::MAX);
        if day != "" && day != "unset" && pid == u64::MAX {
            let mut cp_entry = Item::new();
            info!("Found entry that we want to materialize: {}", entry.0);
            let all_days = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
            let tmp_day = all_days.iter().position(|&r| r == day).unwrap() as u64;
            let ts = week_start
                + (60 * 60 * 24) * tmp_day
                + entry.1.safe_u64("time", 0) % (60 * 60 * 24);
            cp_entry.set_u64("time", ts);
            cp_entry.ids.insert("parent_id".to_string(), *entry.0);
            cp_entry
                .strs
                .insert("day_of_the_week".to_string(), "unset".to_string());

            let mut skip = false;
            for tmp__ in &all_jobs.map {
                if tmp__.1.safe_u64("time", 0) == cp_entry.safe_u64("time", 0)
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
        srv.rw.set_item("job", &ent.clone(), false).await;
    }

    //write_data(srv.deref_mut());
    HttpResponse::Ok().body(
        serde_json::to_string(&ProcessResult {
            succeeded: true,
            error: "".to_string(),
        })
        .unwrap(),
    )
}

pub async fn equestrian_pay_find_broken_payments(
    mut srv: &mut crate::state::data::Data,
    user: Identity,
    query: &str,
) -> HttpResponse {
    let usr = get_user(&mut srv, user.id().unwrap()).await;

    if !check_role(&mut srv, &usr, "admin").await {
        return HttpResponse::Unauthorized().into();
    }

    info!("Query: {}", query);

    HttpResponse::Ok().into()
}

pub async fn equestrian_event_subscribe(
    mut srv: &mut crate::state::data::Data,
    user: Identity,
    query: &str,
) -> HttpResponse {
    let usr = get_user(&mut srv, user.id().unwrap()).await;

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    struct EventSubscribe {
        #[serde(default = "unset_id")]
        pub id: u64,

        #[serde(default = "unset_id")]
        pub event_id: u64,
    }

    let params = web::Query::<EventSubscribe>::from_query(query).unwrap();

    if params.event_id == u64::MAX {
        return HttpResponse::BadRequest().into();
    }

    let mut usr_id = params.id;
    if usr_id == u64::MAX {
        usr_id = usr.as_ref().unwrap().id;
    }

    if usr_id != usr.as_ref().unwrap().id && !check_role(&mut srv, &usr, "admin").await {
        return HttpResponse::Unauthorized().into();
    }

    let itm = srv.rw.get_item("event", params.event_id).await;
    if itm == None {
        return HttpResponse::BadRequest().into();
    }

    info!(
        "Subscribing user {} for event {}...",
        &usr_id, params.event_id
    );
    let mut new_itm = itm.as_ref().unwrap().clone();
    if !new_itm.strstrs.contains_key("participants") {
        new_itm
            .strstrs
            .insert("participants".to_string(), HashMap::new());
    }

    let mut ps = new_itm.safe_strstr("participants", &HashMap::new());
    if !ps.contains_key(&usr_id.to_string()) {
        ps.insert(usr_id.to_string(), "".to_string());
        info!("Inserted");
    }
    new_itm.set_strstr("participants", &ps);

    srv.rw.set_item("event", &new_itm, true).await;
    info!("Subscribed user {} for event {}", &usr_id, params.event_id);

    HttpResponse::Ok().into()
}

pub async fn equestrian_pay_deactivate_expired_payments(
    mut srv: &mut crate::state::data::Data,
    user: Identity,
    _query: &str,
) -> HttpResponse {
    let usr = get_user(&mut srv, user.id().unwrap()).await;
    let now_time = chrono::Local::now().timestamp() as u64;

    if !check_role(&mut srv, &usr, "admin").await {
        return HttpResponse::Unauthorized().into();
    }

    info!("Deactivate expired payments");

    let mut updated_payments: Vec<Item> = Vec::new();
    let jobs = srv.rw.get_all_items("job", "id", "").await;
    let payments = srv.rw.get_all_items("payment", "id", "").await;
    for pay in &payments.map {
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
            .map
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
        srv.rw.set_item("payment", &pay, false).await;
    }

    HttpResponse::Ok().into()
}

