use std::path::Path;
use std::process::Command;
use std::io::Write;
use std::fs::File;
use std::fs;
use std::env;
use std::thread;
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
    let pickle = dir.display().to_string() + "/token.pickle";
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
        .arg("-pickle")
        .arg(pickle)
        .arg(if add { "-add" } else { "-delete" })
        .arg("-add-name")
        .arg(name)
        .arg("-add-date-time")
        .arg(date_time)
        .spawn()
        .expect("Failed to sync with Google");
    info!("Synchronization is done");
}

pub fn init_google(srv: &crate::server::data::Data) -> String {

    if !srv.settings.clone().safe_bool("sync_google_cal", false) ||
       srv.settings.clone().safe_str("sync_google_creds", "") == "" ||
       srv.settings.clone().safe_str("sync_google_email", "") == "" ||
       srv.settings.clone().safe_str("sync_google_cal_name", "") == "" {
        info!("Don't sync with google");
        return "no sync".to_string();
    }

    /* Put credentials to json file */
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    let creds = dir.display().to_string() + "/credentials.json";
    let pickle = dir.display().to_string() + "/token.pickle";

    if !Path::new(&pickle).exists() {
        return "no token".to_string();
    }

    let mut file = File::create(creds.clone()).unwrap();
    write!(file, "{}", srv.settings.str_params["sync_google_creds"].clone());

    info!("Syncing entry with Google...");
    /* Run google calendar sync */
    let res = Command::new(srv.py_path.clone())
        .current_dir(srv.gc_path.clone())
        .arg("-m")
        .arg("igc")
        .arg("-e")
        .arg(srv.settings.str_params["sync_google_email"].clone())
        .arg("-c")
        .arg(srv.settings.str_params["sync_google_cal_name"].clone())
        .arg("-creds")
        .arg(creds)
        .arg("-pickle")
        .arg(pickle)
        .arg("-init")
        .output()
        .expect("Failed to sync with Google");
    info!("Initialization of Google is done");
    return String::from_utf8(res.stdout).unwrap();
}

pub fn auth_google(srv: &crate::server::data::Data) -> String {

    if !srv.settings.clone().safe_bool("sync_google_cal", false) ||
       srv.settings.clone().safe_str("sync_google_creds", "") == "" {
        info!("Don't auth with google");
        return "no auth".to_string();
    }

    /* Put credentials to json file */
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    let creds = dir.display().to_string() + "/credentials.json";
    let pickle = dir.display().to_string() + "/token.pickle";

    if Path::new(&pickle).exists() {
        return "token exists".to_string();
    }

    let mut file = File::create(creds.clone()).unwrap();
    write!(file, "{}", srv.settings.str_params["sync_google_creds"].clone());

    info!("Authentication with Google...");
    /* Run google calendar sync */
    let res = Command::new(srv.py_path.clone())
        .current_dir(srv.gc_path.clone())
        .arg("-m")
        .arg("igc")
        .arg("-flow-start")
        .arg("-flow-url")
        .arg(dir.display().to_string() + "/flow.url")
        .arg("-flow-backlink")
        .arg("http://localhost:8081/setting/gcal_auth")
        .arg("-creds")
        .arg(creds)
        .arg("-pickle")
        .arg(pickle)
        .spawn();
    info!("Flow is running");

    for _i in 1..10 {
      if Path::new(&(dir.display().to_string() + "/flow.url")).exists() {
         let s = fs::read_to_string(&(dir.display().to_string() + "/flow.url")).unwrap();
         info!("URL provided");
         return s;
      }
      thread::sleep_ms(5000);
   }

   info!("Flow timed out");


    return "running".to_string();
}

pub fn auth_google_end(srv: &crate::server::data::Data,
                       code: String) -> String {

    if !srv.settings.clone().safe_bool("sync_google_cal", false) ||
       srv.settings.clone().safe_str("sync_google_creds", "") == "" {
        info!("Don't auth with google");
        return "no auth".to_string();
    }

    /* Put credentials to json file */
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    let creds = dir.display().to_string() + "/credentials.json";
    let pickle = dir.display().to_string() + "/token.pickle";

    if Path::new(&pickle).exists() {
        return "token exists".to_string();
    }

    let mut file = File::create(creds.clone()).unwrap();
    write!(file, "{}", srv.settings.str_params["sync_google_creds"].clone());

    info!("Finish Authentication with Google...");
    /* Run google calendar sync */
    let res = Command::new(srv.py_path.clone())
        .current_dir(srv.gc_path.clone())
        .arg("-m")
        .arg("igc")
        .arg("-flow-end")
        .arg("-flow-token")
        .arg(code)
        .arg("-creds")
        .arg(creds)
        .arg("-pickle")
        .arg(pickle)
        .spawn();
    info!("Started OAuth saver");

    thread::sleep_ms(5000);

    return "running".to_string();
}
