
use crate::handler::equestrian::equestrian_job_sync;

pub fn call_item_route(srv: & mut crate::state::data::Data, hndl: &str, collection: &str, id: u64, del: bool) {
	match hndl {
	    "equestrian_job_sync" => equestrian_job_sync(srv, collection, id, del),
	    &_ => { }
	}
}
