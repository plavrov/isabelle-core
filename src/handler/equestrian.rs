
use crate::init_google;
use crate::notif::gcal::sync_with_google;
use chrono::NaiveDateTime;
use now::DateTimeNow;
use chrono::DateTime;
use isabelle_dm::data_model::item::Item;
use chrono::Utc;
use crate::notif::email::send_email;
use log::info;

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


pub fn equestrian_job_sync(srv: & mut crate::state::data::Data, collection: &str, id: u64, del: bool) {
	if collection != "job" {
		info!("Not job");
		return;
	}

	let j = srv.itm["job"].get(id);
	if j == None {
		info!("No job");
		return;
	}
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
            	info!("Found user: {}", user.as_ref().unwrap().safe_str("firstname", ""));
                let target_email = user.as_ref().unwrap().safe_str(em, "");
                if user.as_ref().unwrap().safe_bool("notify_training_email", false) &&
                   target_email != "" {
                   	info!("Target email found");
                   	if del {
                   		send_email(&srv,
	                        &target_email,
	                        "Schedule changed",
	                        "The schedule entry has been removed. Please review your new schedule");
                   	} else {
	                    send_email(&srv,
	                        &target_email,
	                        "Schedule changed",
	                        &format!("Please review changes for the following entry:\n{}{}",
	                        srv.public_url.clone() + "/job/edit?id=",
	                        &id.to_string()));
	                }
	            } else {
	            	info!("Target email not found");
	            }
            }
        }
    }

    init_google(&srv);
    sync_with_google(&srv,
                     if del { false } else { true },
                     eventname(&srv, &job),
                     entry2datetimestr(&job));
}