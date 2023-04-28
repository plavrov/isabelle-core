
mod data_model;
mod server;
use actix_identity::Identity;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, cookie::Key, cookie::SameSite};
use actix_web::web::Data;
use crate::server::state::*;
use serde::{Deserialize, Serialize};

use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_identity::IdentityMiddleware;
use actix_cors::Cors;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn user_list(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpUser {
        pub id: u64,
        pub firstname: String,
        pub surname: String,
    }

    #[derive(Deserialize, Debug, Serialize, Clone)]
    struct TmpUserList {
        pub users: Vec<TmpUser>
    }

    let _srv = data.server.lock().unwrap();

    let mut lst = TmpUserList {
        users: Vec::new(),
    };

    for el in &_srv.users {
        lst.users.push(TmpUser {
            id: *el.0 as u64,
            firstname: el.1.firstname.clone(),
            surname: el.1.surname.clone(),
        });
    }

    web::Json(lst)
}

// The secret key would usually be read from a configuration file/environment variables.
fn get_secret_key() -> Key {
    return Key::generate();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    let data = Data::new(State::new());
    let secret_key = get_secret_key();
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Cors::permissive())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_key.clone()
                )
                .cookie_same_site(SameSite::None)
                .cookie_secure(false)
                .cookie_http_only(false)
                .build(),
            )
            .route("/list", web::get().to(user_list))
            .route("/hello", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
