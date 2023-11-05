use crate::handler::route::call_otp_hook;
use crate::server::user_control::*;
use crate::state::state::*;
use crate::state::store::Store;
use crate::util::crypto::get_otp_code;
use crate::util::crypto::verify_password;
use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures_util::TryStreamExt;
use isabelle_dm::data_model::process_result::ProcessResult;
use isabelle_dm::transfer_model::detailed_login_user::DetailedLoginUser;
use isabelle_dm::transfer_model::login_user::LoginUser;
use log::{error, info};
use std::collections::HashMap;

pub async fn gen_otp(
    _user: Option<Identity>,
    data: web::Data<State>,
    mut payload: Multipart,
    _req: HttpRequest,
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
            }
        }
    }

    let mut srv = data.server.lock().unwrap();
    info!("User name: {}", lu.username.clone());
    let usr = get_user(&mut srv, lu.username.clone()).await;

    if usr == None {
        info!("No user {} found, couldn't log in", lu.username.clone());
        return web::Json(ProcessResult {
            succeeded: false,
            error: "Invalid login".to_string(),
        });
    } else {
        let mut new_usr_itm = srv
            .rw
            .get_item("user", usr.clone().unwrap().id)
            .await
            .unwrap();
        new_usr_itm.set_str("otp", &get_otp_code());
        srv.rw.set_item("user", &new_usr_itm, false).await;

        let routes = srv
            .rw
            .get_internals()
            .await
            .safe_strstr("otp_hook", &HashMap::new());
        for route in routes {
            call_otp_hook(&mut srv, &route.1, new_usr_itm.clone()).await;
        }
    }

    return web::Json(ProcessResult {
        succeeded: true,
        error: "".to_string(),
    });
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

    let mut srv = data.server.lock().unwrap();
    info!("User name: {}", lu.username.clone());
    let usr = get_user(&mut srv, lu.username.clone()).await;

    if usr == None {
        info!("No user {} found, couldn't log in", lu.username.clone());
        return web::Json(ProcessResult {
            succeeded: false,
            error: "Invalid login/password".to_string(),
        });
    } else {
        let itm_real = usr.unwrap();

        clear_otp(&mut srv, lu.username.clone()).await;

        let pw = itm_real.safe_str("password", "");
        let otp = itm_real.safe_str("otp", "");
        if (pw != "" && verify_password(&lu.password, &pw)) || (otp != "" && lu.password == otp) {
            Identity::login(&req.extensions(), itm_real.safe_str("email", "")).unwrap();
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
    let mut srv = data.server.lock().unwrap();

    let mut user: DetailedLoginUser = DetailedLoginUser {
        username: "".to_string(),
        id: 0,
        role: Vec::new(),
        site_name: "".to_string(),
        site_logo: "".to_string(),
        licensed_to: "".to_string(),
    };

    user.site_name = srv
        .rw
        .get_settings()
        .await
        .clone()
        .safe_str("site_name", "");
    if user.site_name == "" {
        user.site_name = srv
            .rw
            .get_internals()
            .await
            .safe_str("default_site_name", "Isabelle");
    }

    user.site_logo = srv
        .rw
        .get_settings()
        .await
        .clone()
        .safe_str("site_logo", "");
    if user.site_logo == "" {
        user.site_logo = srv
            .rw
            .get_internals()
            .await
            .safe_str("default_site_logo", "/logo.png");
    }
    info!("Site logo: {}", user.site_logo);

    user.licensed_to = srv
        .rw
        .get_settings()
        .await
        .clone()
        .safe_str("licensed_to", "");
    if user.licensed_to == "" {
        user.licensed_to = srv
            .rw
            .get_internals()
            .await
            .safe_str("default_licensed_to", "end user");
    }

    if _user.is_none() || !srv.has_collection("user") {
        info!("No user or user database");
        return web::Json(user);
    }

    let role_is = srv
        .rw
        .get_internals()
        .await
        .safe_str("user_role_prefix", "role_is_");
    let all_users = srv.rw.get_all_items("user").await;
    for item in &all_users {
        if item.1.strs.contains_key("email")
            && item.1.strs["email"] == _user.as_ref().unwrap().id().unwrap()
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
