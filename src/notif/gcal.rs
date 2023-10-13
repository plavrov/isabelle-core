use std::io::Write;
use std::fs::File;
use std::env;
use std::process::Command;
use log::{info};

pub fn sync_with_google(srv: &crate::server::data::Data,
                        add: bool,
                        name: String,
                        date_time: String) {

    /* Put credentials to json file */
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    let creds = dir.display().to_string() + "/credentials.json";
    let mut file = File::create(creds.clone()).unwrap();
    write!(file, "{}", srv.settings.str_params["sync_google_creds"].clone());

    info!("Syncing entry with Google...");
    /* Run google calendar sync */
    Command::new("python3")
        .current_dir(srv.gc_path.clone())
        .env("PATH", "/opt/homebrew/opt/binutils/bin".to_owned() +
                     ":/Users/mmenshikov/.cargo/bin" +
                     ":/opt/homebrew/opt/llvm/bin" +
                     ":/Users/mmenshikov/.local/share/gem/ruby/3.1.0/bin" +
                     ":/opt/homebrew/opt/openjdk/bin" +
                     ":/opt/homebrew/bin" +
                     ":/usr/local/bin" +
                     ":/System/Cryptexes/App/usr/bin" +
                     ":/usr/bin" +
                     ":/bin" +
                     ":/usr/sbin" +
                     ":/sbin" +
                     ":/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/local/bin" +
                     ":/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/bin" +
                     ":/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/appleinternal/bin" +
                     ":/opt/X11/bin" +
                     ":/Library/Apple/usr/bin" +
                     ":/Library/TeX/texbin" +
                     ":/Applications/Wireshark.app/Contents/MacOS")
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
