/*
 * Isabelle project
 *
 * Copyright 2023-2024 Maxim Menshikov
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the “Software”),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */
use isabelle_plugin_api::api::WebResponse;
use crate::state::store::Store;
use crate::State;
use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use log::info;
use std::collections::HashMap;
use futures_util::TryStreamExt;
use crate::server::user_control::*;

fn conv_response(resp: WebResponse) -> HttpResponse {
    match resp {
        WebResponse::Ok => {
            return HttpResponse::Ok().into();
        }
        WebResponse::OkData(_text) => {
            return HttpResponse::Ok().body(
                serde_json::to_string(&ProcessResult {
                    succeeded: true,
                    error: "".to_string(),
                })
                .unwrap(),
            )
        }
        WebResponse::NotFound => {
            return HttpResponse::NotFound().into();
        }
        WebResponse::Unauthorized => {
            return HttpResponse::Unauthorized().into();
        }
        WebResponse::BadRequest => {
            return HttpResponse::BadRequest().into();
        }
        WebResponse::Forbidden => {
            return HttpResponse::Forbidden().into();
        }
    }
}

pub async fn call_item_pre_edit_hook(
    srv: &mut crate::state::data::Data,
    hndl: &str,
    user: &Option<Item>,
    collection: &str,
    old_itm: Option<Item>,
    itm: &mut Item,
    del: bool,
    merge: bool,
) -> ProcessResult {
    for hook in &srv.item_pre_edit_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return hook.1(&srv.plugin_api, user, collection, old_itm, itm, del, merge);
        }
    }

    match hndl {
        &_ => {
            return ProcessResult {
                succeeded: true,
                error: "".to_string(),
            };
        }
    }
}

pub async fn call_item_post_edit_hook(
    srv: &mut crate::state::data::Data,
    hndl: &str,
    collection: &str,
    id: u64,
    del: bool,
) {
    for hook in &srv.item_post_edit_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return hook.1(&srv.plugin_api, collection, id, del);
        }
    }

    match hndl {
        &_ => {}
    }
}

pub async fn call_itm_auth_hook(
    srv: &mut crate::state::data::Data,
    hndl: &str,
    user: &Option<Item>,
    collection: &str,
    id: u64,
    new_item: Option<Item>,
    del: bool,
) -> bool {
    for hook in &srv.item_auth_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return hook.1(&srv.plugin_api, user, collection, id, new_item, del);
        }
    }

    match hndl {
        &_ => return false,
    }
}

pub async fn call_itm_list_filter_hook(
    srv: &mut crate::state::data::Data,
    hndl: &str,
    user: &Option<Item>,
    collection: &str,
    context: &str,
    map: &mut HashMap<u64, Item>,
) {
    for hook in &srv.item_list_filter_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return hook.1(&srv.plugin_api, user, collection, context, map);
        }
    }
    match hndl {
        &_ => {}
    }
}

pub async fn call_url_route(
    srv: &mut crate::state::data::Data,
    user: Identity,
    hndl: &str,
    query: &str,
) -> HttpResponse {
    let usr : Option<Item> =get_user(srv, user.id().unwrap()).await;

    for hook in &srv.url_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return conv_response(hook.1(&srv.plugin_api, &usr, query));
        }
    }

    match hndl {
        &_ => {
            return HttpResponse::NotFound().into();
        }
    }
}

pub async fn url_route(
    user: Identity,
    data: actix_web::web::Data<State>,
    req: HttpRequest,
) -> HttpResponse {
    let srv_lock = data.server.lock();
    let mut srv = srv_lock.borrow_mut();
    let routes = srv
        .rw
        .get_internals()
        .await
        .safe_strstr("extra_route", &HashMap::new());

    info!("Custom URL: {}", req.path());

    for route in routes {
        let parts: Vec<&str> = route.1.split(":").collect();
        if parts[0] == req.path() {
            info!("Call custom route {}", parts[2]);
            return call_url_route(&mut srv, user, parts[2], req.query_string()).await;
        }
    }

    HttpResponse::NotFound().into()
}

pub async fn call_url_unprotected_route(
    srv: &mut crate::state::data::Data,
    user: Option<Identity>,
    hndl: &str,
    query: &str,
) -> HttpResponse {
    let mut usr : Option<Item> = None;

    if user.is_none() {
        usr = get_user(srv, user.unwrap().id().unwrap()).await;
    }

    for hook in &srv.unprotected_url_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return conv_response(hook.1(&srv.plugin_api, &usr, query));
        }
    }

    match hndl {
        &_ => {
            return HttpResponse::NotFound().into();
        }
    }
}

pub async fn call_url_unprotected_post_route(
    mut srv: &mut crate::state::data::Data,
    user: Option<Identity>,
    hndl: &str,
    query: &str,
    mut payload: Multipart,
) -> HttpResponse {
    let mut usr : Option<Item> = None;

    if user.is_none() {
        usr = get_user(&mut srv, user.unwrap().id().unwrap()).await;
    }

    let mut post_itm = Item::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Ok(Some(chunk)) = field.try_next().await {
            let data = chunk;

            if field.name() == "item" {
                let v = &data.to_vec();
                let strv = std::str::from_utf8(v).unwrap_or("{}");
                let new_itm: Item = serde_json::from_str(strv).unwrap_or(Item::new());
                post_itm.merge(&new_itm);
            }
        }
    }

    for hook in &srv.unprotected_url_post_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return conv_response(hook.1(&srv.plugin_api, &usr, query, &post_itm));
        }
    }

    match hndl {
        &_ => {
            return HttpResponse::NotFound().into();
        }
    }
}

pub async fn url_unprotected_route(
    user: Option<Identity>,
    data: actix_web::web::Data<State>,
    req: HttpRequest,
) -> HttpResponse {
    let srv_lock = data.server.lock();
    let mut srv = srv_lock.borrow_mut();
    let routes = srv
        .rw
        .get_internals()
        .await
        .safe_strstr("extra_unprotected_route", &HashMap::new());

    info!("Custom unprotected URL: {}", req.path());

    for route in routes {
        let parts: Vec<&str> = route.1.split(":").collect();
        if parts[0] == req.path() {
            info!("Call custom route {}", parts[2]);
            return call_url_unprotected_route(&mut srv, user, parts[2], req.query_string()).await;
        }
    }

    HttpResponse::NotFound().into()
}

pub async fn url_unprotected_post_route(
    user: Option<Identity>,
    data: actix_web::web::Data<State>,
    req: HttpRequest,
    payload: Multipart,
) -> HttpResponse {
    let srv_lock = data.server.lock();
    let mut srv = srv_lock.borrow_mut();

    let routes = srv
        .rw
        .get_internals()
        .await
        .safe_strstr("extra_unprotected_route", &HashMap::new());

    info!("Custom unprotected post URL: {}", req.path());

    for route in routes {
        let parts: Vec<&str> = route.1.split(":").collect();
        if parts[0] == req.path() {
            info!("Call custom route {}", parts[2]);
            return call_url_unprotected_post_route(
                &mut srv,
                user,
                parts[2],
                req.query_string(),
                payload,
            )
            .await;
        }
    }

    HttpResponse::NotFound().into()
}

pub async fn call_collection_read_hook(
    data: &mut crate::state::data::Data,
    hndl: &str,
    collection: &str,
    itm: &mut Item) -> bool {

    for hook in &data.collection_read_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return hook.1(&data.plugin_api, collection, itm);
        }
    }
    match hndl {
        _ => {
            return false;
        }
    }
}

pub async fn call_otp_hook(srv: &mut crate::state::data::Data, hndl: &str, itm: Item) {
    for hook in &srv.call_otp_hook {
        if hndl == hook.0 {
            info!("Calling hook {}", hook.0);
            return hook.1(&srv.plugin_api, &itm);
        }
    }
    match hndl {
        _ => {}
    }
}
