use chrono::DateTime;
use chrono::NaiveDateTime;
use std::ops::Deref;
use isabelle_dm::data_model::item::Item;
use crate::state::data_rw::*;
use crate::state::state::*;
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use log::{info};
use now::DateTimeNow;
use serde::{Deserialize, Serialize};

use std::ops::DerefMut;

use crate::server::user_control::*;

pub fn eventname(srv: &crate::state::data::Data, sch: &Item) -> String {
    let teacher_id = sch.safe_id("teacher", 0);
    if teacher_id == 0 {
        "Training".to_string()
    } else {
        "Training with ".to_owned()
            + &srv.itm["user"].get(teacher_id).unwrap().safe_str("firstname", "<unknown>")
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

    HttpResponse::Ok()
}
