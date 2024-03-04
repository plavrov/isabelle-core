use crate::state::merger::merge_database;
use crate::state::store::Store;
mod handler;
mod notif;
mod server;
mod state;
mod util;

use crate::handler::route::url_route;
use crate::handler::route::url_unprotected_post_route;
use crate::handler::route::url_unprotected_route;
use crate::notif::gcal::init_google;
use crate::server::itm::*;
use crate::server::login::*;
use std::collections::HashMap;

use crate::server::setting::*;

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

fn session_middleware(pub_fqdn: String) -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
        .session_lifecycle(BrowserSession::default())
        .cookie_same_site(SameSite::None)
        .cookie_path("/".into())
        .cookie_name(String::from("isabelle-cookie"))
        .cookie_domain(Some(pub_fqdn.into()))
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
    let mut db_url: String = "mongodb://127.0.0.1:27017".to_string();
    let mut pub_path: String = "http://localhost:8081".to_string();
    let mut pub_fqdn: String = "localhost".to_string();
    let mut database_name: String = "isabelle".to_string();
    let mut port: u16 = 8090;
    let mut gc_next = false;
    let mut py_next = false;
    let mut data_next = false;
    let mut pub_next = false;
    let mut pub_fqdn_next = false;
    let mut port_next = false;
    let mut db_url_next = false;
    let mut database_name_next = false;
    let mut first_run = false;

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
        } else if pub_fqdn_next {
            pub_fqdn = arg.clone();
            pub_fqdn_next = false;
        } else if port_next {
            port = arg.parse().unwrap();
            port_next = false;
        } else if db_url_next {
            db_url = arg.parse().unwrap();
            db_url_next = false;
        } else if database_name_next {
            database_name = arg.parse().unwrap();
            database_name_next = false;
        }

        if arg == "--gc-path" {
            gc_next = true;
        } else if arg == "--py-path" {
            py_next = true;
        } else if arg == "--data-path" {
            data_next = true;
        } else if arg == "--pub-url" {
            pub_next = true;
        } else if arg == "--pub-fqdn" {
            pub_fqdn_next = true;
        } else if arg == "--db-url" {
            db_url_next = true;
        } else if arg == "--database" {
            database_name_next = true;
        } else if arg == "--first-run" {
            first_run = true;
        }
    }

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let mut new_routes: HashMap<String, String> = HashMap::new();
    let mut new_unprotected_routes: HashMap<String, String> = HashMap::new();
    let state = State::new();
    {
        let mut srv = state.server.lock().unwrap();
        {
            (*srv.deref_mut()).rw.database_name = database_name.clone();
            (*srv.deref_mut()).file_rw.connect(&data_path, "").await;
            (*srv.deref_mut()).rw.connect(&db_url, &data_path).await;
            (*srv.deref_mut()).gc_path = gc_path.to_string();
            (*srv.deref_mut()).py_path = py_path.to_string();
            (*srv.deref_mut()).data_path = data_path.to_string();
            (*srv.deref_mut()).public_url = pub_path.to_string();
            (*srv.deref_mut()).port = port;

            (*srv.deref_mut()).init_checks().await;

            info!("Initializing google!");
            let res = init_google(srv.deref_mut()).await;
            info!("Result: {}", res);

            {
                let routes = (*srv.deref_mut())
                    .rw
                    .get_internals()
                    .await
                    .safe_strstr("extra_route", &HashMap::new());
                for route in routes {
                    let parts: Vec<&str> = route.1.split(":").collect();
                    new_routes.insert(parts[0].to_string(), parts[1].to_string());
                    info!("Route: {} : {}", parts[0], parts[1]);
                }
            }
            {
                let routes = (*srv.deref_mut())
                    .rw
                    .get_internals()
                    .await
                    .safe_strstr("extra_unprotected_route", &HashMap::new());
                for route in routes {
                    let parts: Vec<&str> = route.1.split(":").collect();
                    new_unprotected_routes.insert(parts[0].to_string(), parts[1].to_string());
                    info!("Unprotected route: {} : {}", parts[0], parts[1]);
                }
            }
            if first_run {
                let srv_mut = srv.deref_mut();
                merge_database(&mut srv_mut.file_rw, &mut srv_mut.rw).await;
            }
        }
    }

    if first_run {
        return Ok(());
    }

    let data = Data::new(state);
    info!("Starting server");
    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(data.clone())
            .wrap(Cors::permissive())
            .wrap(IdentityMiddleware::default())
            .wrap(session_middleware(pub_fqdn.clone()))
            .route("/itm/edit", web::post().to(itm_edit))
            .route("/itm/del", web::post().to(itm_del))
            .route("/itm/list", web::get().to(itm_list))
            .route("/login", web::post().to(login))
            .route("/gen_otp", web::post().to(gen_otp))
            .route("/logout", web::post().to(logout))
            .route("/is_logged_in", web::get().to(is_logged_in))
            .route("/setting/edit", web::post().to(setting_edit))
            .route("/setting/list", web::get().to(setting_list))
            .route("/setting/gcal_auth", web::post().to(setting_gcal_auth))
            .route(
                "/setting/gcal_auth_end",
                web::post().to(setting_gcal_auth_end),
            );
        for route in &new_routes {
            if route.1 == "post" {
                app = app.route(route.0, web::post().to(url_route))
            } else if route.1 == "get" {
                app = app.route(route.0, web::get().to(url_route))
            }
        }
        for route in &new_unprotected_routes {
            if route.1 == "post" {
                app = app.route(route.0, web::post().to(url_unprotected_post_route))
            } else if route.1 == "get" {
                app = app.route(route.0, web::get().to(url_unprotected_route))
            }
        }
        app
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
