use std::process::Command;
use std::io::Write;
use std::fs::File;
use std::env;
use log::{info};

pub fn sync_with_google(srv: &crate::server::data::Data,
                        add: bool,
                        name: String,
                        date_time: String) {

    if !srv.settings.clone().safe_bool("sync_google_cal", false) ||
       srv.settings.clone().safe_str("sync_google_creds", "") == "" ||
       srv.settings.clone().safe_str("sync_google_email", "") == "" ||
       srv.settings.clone().safe_str("sync_google_cal_name", "") == "" {
        info!("Don't sync with google");
        return;
    }

    /* Put credentials to json file */
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    let creds = dir.display().to_string() + "/credentials.json";
    let mut file = File::create(creds.clone()).unwrap();
    write!(file, "{}", srv.settings.str_params["sync_google_creds"].clone());

    info!("Syncing entry with Google...");
    /* Run google calendar sync */
    Command::new(srv.py_path.clone())
        .current_dir(srv.gc_path.clone())
        .arg("-m")
        .arg("igc")
        .arg("-e")
        .arg(srv.settings.str_params["sync_google_email"].clone())
        .arg("-c")
        .arg(srv.settings.str_params["sync_google_cal_name"].clone())
        .arg("-creds")
        .arg(creds)
        .arg(if add { "-add" } else { "-delete" })
        .arg("-add-name")
        .arg(name)
        .arg("-add-date-time")
        .arg(date_time)
        .spawn()
        .expect("Failed to sync with Google");
    info!("Synchronization is done");
}

pub fn init_google(srv: &crate::server::data::Data) {

    if !srv.settings.clone().safe_bool("sync_google_cal", false) ||
       srv.settings.clone().safe_str("sync_google_creds", "") == "" ||
       srv.settings.clone().safe_str("sync_google_email", "") == "" ||
       srv.settings.clone().safe_str("sync_google_cal_name", "") == "" {
        info!("Don't sync with google");
        return;
    }

    /* Put credentials to json file */
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    let creds = dir.display().to_string() + "/credentials.json";
    let mut file = File::create(creds.clone()).unwrap();
    write!(file, "{}", srv.settings.str_params["sync_google_creds"].clone());

    info!("Syncing entry with Google...");
    /* Run google calendar sync */
    Command::new(srv.py_path.clone())
        .current_dir(srv.gc_path.clone())
        .arg("-m")
        .arg("igc")
        .arg("-e")
        .arg(srv.settings.str_params["sync_google_email"].clone())
        .arg("-c")
        .arg(srv.settings.str_params["sync_google_cal_name"].clone())
        .arg("-creds")
        .arg(creds)
        .arg("-init")
        .spawn()
        .expect("Failed to sync with Google");
    info!("Initialization of Google is done");
}
