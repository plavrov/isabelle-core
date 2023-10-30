use crate::server::user_control::*;
use crate::state::state::*;
use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use isabelle_dm::data_model::process_result::ProcessResult;
use isabelle_dm::data_model::login_user::LoginUser;
use log::{error, info};
use serde::{Deserialize, Serialize};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

pub fn verify_password(pw: &str, pw_hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(&pw_hash);
    Argon2::default().verify_password(pw.as_bytes(), &parsed_hash.unwrap()).is_ok()
}

pub fn get_new_salt() -> String {
    let salt = SaltString::generate(&mut OsRng);
    return salt.to_string();
}

pub fn get_password_hash(pw: &str, salt: &str) -> String {
    let argon2 = Argon2::default();

    let saltstr = SaltString::from_b64(&salt).unwrap();
    let password_hash = argon2.hash_password(pw.as_bytes(), saltstr.as_salt());

    return password_hash.unwrap().serialize().as_str().to_string();
}

pub async fn login(
    _user: Option<Identity>,
    data: web::Data<State>,
    mut payload: Multipart,
    req: HttpRequest,
) -> impl Responder {
    let mut lu = LoginUser {
        username: "".to_string(),
        password: "".to_string(),
    };

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Ok(Some(chunk)) = field.try_next().await {
            let data = chunk;

            if field.name() == "username" {
                lu.username = std::str::from_utf8(&data.to_vec()).unwrap().to_string();
            } else if field.name() == "password" {
                lu.password = std::str::from_utf8(&data.to_vec()).unwrap().to_string();
            }
        }
    }

    let srv = data.server.lock().unwrap();
    info!("User name: {}", lu.username.clone());
    let usr = get_user(&srv, lu.username.clone());

    if usr == None {
        info!("No user {} found, couldn't log in", lu.username.clone());
        return web::Json(ProcessResult {
            succeeded: false,
            error: "Invalid login/password".to_string(),
        });
    } else {
        let itm_real = usr.unwrap();

        if itm_real.strs.contains_key("password")
            && itm_real.safe_str("password", "") == lu.password
        {
            Identity::login(&req.extensions(), lu.username.clone()).unwrap();
            info!("Logged in as {}", lu.username);
        } else {
            error!("Invalid password for {}", lu.username);
            return web::Json(ProcessResult {
                succeeded: false,
                error: "Invalid login/password".to_string(),
            });
        }
    }

    return web::Json(ProcessResult {
        succeeded: true,
        error: "".to_string(),
    });
}

pub async fn logout(
    _user: Identity,
    _data: web::Data<State>,
    _request: HttpRequest,
) -> impl Responder {
    _user.logout();
    info!("Logged out");

    HttpResponse::Ok()
}

pub async fn is_logged_in(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    let srv = data.server.lock().unwrap();

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct LoginUser {
        pub username: String,
        pub id: u64,
        pub role: Vec<String>,
        pub site_name: String,
        pub site_logo: String,
        pub licensed_to: String,
    }

    let mut user: LoginUser = LoginUser {
        username: "".to_string(),
        id: 0,
        role: Vec::new(),
        site_name: "".to_string(),
        site_logo: "".to_string(),
        licensed_to: "".to_string(),
    };

    user.site_name = srv.settings.clone().safe_str("site_name", "");
    if user.site_name == "" {
        user.site_name = srv.internals.safe_str("default_site_name", "Isabelle");
    }

    user.site_logo = srv.settings.clone().safe_str("site_logo", "");
    if user.site_logo == "" {
        user.site_logo = srv.internals.safe_str("default_site_logo", "/logo.png");
    }
    info!("Site logo: {}", user.site_logo);

    user.licensed_to = srv.settings.clone().safe_str("licensed_to", "");
    if user.licensed_to == "" {
        user.licensed_to = srv.internals.safe_str("default_licensed_to", "end user");
    }

    if _user.is_none() || !srv.itm.contains_key("user") {
        info!("No user or user database");
        return web::Json(user);
    }

    let role_is = srv.internals.safe_str("user_role_prefix", "role_is_");
    for item in srv.itm["user"].get_all() {
        if item.1.strs.contains_key("login")
            && item.1.strs["login"] == _user.as_ref().unwrap().id().unwrap()
        {
            user.username = _user.as_ref().unwrap().id().unwrap();
            user.id = *item.0;
            for bp in &item.1.bools {
                if bp.0.starts_with(&role_is) {
                    user.role.push(bp.0[8..].to_string());
                }
            }
            break;
        }
    }

    web::Json(user)
}
