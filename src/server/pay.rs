use crate::state::state::*;
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use log::{info};
use crate::server::user_control::*;

pub async fn pay_find_broken_payments(
    _user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    let srv = data.server.lock().unwrap();
    let usr = get_user(&srv, _user.id().unwrap());

    if check_role(usr, "admin") {
        return HttpResponse::Unauthorized();
    }

    info!("Query: {}", &req.query_string());

    HttpResponse::Ok()
}
