mod notif;
mod server;
mod state;

use crate::notif::gcal::init_google;
use crate::server::item::*;
use crate::server::login::*;
use crate::server::schedule::*;
use crate::server::setting::*;
use crate::state::data_rw::*;
use crate::state::state::*;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::config::{BrowserSession, CookieContentSecurity};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::web::Data;
use actix_web::{cookie::Key, cookie::SameSite, web, App, HttpServer};
use log::info;
use std::env;
use std::ops::DerefMut;

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
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
    let mut gc_path: String = "".to_string();
    let mut py_path: String = "".to_string();
    let mut data_path: String = "sample-data".to_string();
    let mut pub_path: String = "http://localhost:8081".to_string();
    let mut port: u16 = 8090;
    let mut gc_next = false;
    let mut py_next = false;
    let mut data_next = false;
    let mut pub_next = false;
    let mut port_next = false;

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
        } else if port_next {
            port = arg.parse().unwrap();
            port_next = false;
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
            (*srv.deref_mut()).port = port;

            info!("Initializing google!");
            let res = init_google(srv.deref_mut());
            info!("Result: {}", res);
        }
    }

    let data = Data::new(state);
    info!("Starting server");
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Cors::permissive())
            .wrap(IdentityMiddleware::default())
            .wrap(session_middleware())
            .route("/item/edit", web::post().to(item_edit))
            .route("/item/del", web::post().to(item_del))
            .route("/item/done", web::post().to(item_done))
            .route("/item/list", web::get().to(item_list))
            .route("/schedule/edit", web::post().to(schedule_entry_edit))
            .route("/schedule/del", web::post().to(schedule_entry_del))
            .route("/schedule/list", web::get().to(schedule_entry_list))
            .route("/schedule/done", web::post().to(schedule_entry_done))
            .route("/schedule/paid", web::post().to(schedule_entry_paid))
            .route(
                "/schedule/materialize",
                web::post().to(schedule_materialize),
            )
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/is_logged_in", web::get().to(is_logged_in))
            .route("/setting/edit", web::post().to(setting_edit))
            .route("/setting/list", web::get().to(setting_list))
            .route("/setting/gcal_auth", web::post().to(setting_gcal_auth))
            .route(
                "/setting/gcal_auth_end",
                web::post().to(setting_gcal_auth_end),
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
