extern crate actix_web;
extern crate futures;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
use actix_web::*;
use futures::Future;
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};


mod db;

#[derive(Serialize)]
struct Resp<'a> {
    status: &'a str
}


fn index(_req: HttpRequest<Arc<Mutex<db::Database>>>) -> String {
    "Hello world".to_owned()
}

fn batch_insert(req: HttpRequest<Arc<Mutex<db::Database>>>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let req_clone = req.clone();
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(move|val: db::TicketsInsertRequest| {
            let db: &Arc<Mutex<db::Database>> = req_clone.state();
            db.lock().unwrap().insert_tickets(val.tickets);
            let resp = Resp{status: "success"};
            Ok(httpcodes::HTTPOk.build().json(resp)?/*finish()?*/)  // <- send response
        })
        .responder()
}

fn search(req: HttpRequest<Arc<Mutex<db::Database>>>) ->Box<Future<Item=HttpResponse, Error=Error>> {
    let req_clone = req.clone();
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(move|val: db::SearchRequest| {
            let db: &Arc<Mutex<db::Database>> = req_clone.state();
            let result = db.lock().unwrap().search_flights(&val);
            match result {
                Ok(solutions) => Ok(httpcodes::HTTPOk.build().json(solutions)?),
                Err(error) => {
                    let resp = Resp{status: error};
                    Ok(httpcodes::HTTPOk.build().json(resp)?)
                }
            }
        })
        .responder()
}

fn main() {

    HttpServer::new(
        || Application::with_state( Arc::new(Mutex::new(db::Database {airports: RefCell::new(HashMap::new())})))
            .resource("/batch_insert", |r| r.method(Method::POST).f(batch_insert))
            .resource("/search", |r| r.method(Method::POST).f(search))
            .resource("/", |r| r.f(index))
        )
        .bind("0.0.0.0:8080").unwrap()
        .run();
}