extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate env_logger;
use actix::*;
use actix_web::*;
use futures::{Future, Stream};
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::io;


mod db;

#[derive(Serialize, Debug)]
pub struct Resp {
    status: &'static str
}

impl Actor for db::Database {
    type Context = SyncContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Actor started")
    }
}

impl Message for db::TicketsInsertRequest {
    type Result = Result<bool, io::Error>;
}

impl Message for db::SearchRequest {
    type Result = Result<db::Solutions, Resp>;
}

impl Handler<db::TicketsInsertRequest> for db::Database {
    type Result = Result<bool, io::Error>;

    fn handle(&mut self, msg: db::TicketsInsertRequest, ctx: &mut Self::Context) -> Self::Result {

        info!("{:?}", self.airports);
        info!("inside handler");
        self.insert_tickets(msg.tickets);
        Ok(true)
    }
}

impl Handler<db::SearchRequest> for db::Database {
    type Result = Result<db::Solutions, Resp>;//??

    fn handle(&mut self, msg: db::SearchRequest, ctx: &mut Self::Context) -> Self::Result {
        match self.search_flights(&msg) {
            Ok(solutions) => Ok(solutions),
            Err(error) => {
                Err(Resp { status: "failure" })
            }
        }
    }
}

fn index<'a>(_req: HttpRequest<actix::Addr<actix::Syn, db::Database>>) -> &'a str {
    "Hello world"
}

fn batch_insert(req: HttpRequest<actix::Addr<actix::Syn, db::Database>>) -> Box<Future<Item=HttpResponse, Error=Error>> {

    let db_actor: actix::Addr<actix::Syn, db::Database> = req.state().clone();

    req.concat2()
        .from_err()
        .map(move|body|{
            let res = serde_json::from_slice::<db::TicketsInsertRequest>(&body);
            match res {
                Ok(ticket_insert_req) => Ok(db_actor.send(ticket_insert_req)),
                Err(e) => Err(e)
            }
        }).and_then(|res| {
            match res {
                Ok(insert_res) => Ok(httpcodes::HttpOk.build().json(Resp { status: "success" })?),
                Err(e) => Ok(httpcodes::HttpOk.build().json(Resp { status: "failure" })?)
            }
        }).responder()

}

fn search(req: HttpRequest<actix::Addr<actix::Syn, db::Database>>) ->Box<Future<Item=HttpResponse, Error=Error>> {

    let sr = db::SearchRequest{
        departure_code: "LED".to_owned(),
        arrival_code: "DME".to_owned(),
        departure_time_start: 1509840000,
        departure_time_end: 1509926399
    };

    let a = req.state().send(sr).wait();
    match a {
        Ok(s) => println!("Solutions: {:?}", s),
        Err(e) => println!("Err: {:?}", e)
    }

   /* let db: Arc<Mutex<db::Database>> = req.state().clone();*/
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(move|val: db::SearchRequest| {
//            let db: &Arc<Mutex<db::Database>> = req_clone.state();
//            let result = db.lock().unwrap().search_flights(&val);
           /* match a {
                Ok(solutions) => Ok(httpcodes::HTTPOk.build().json(solutions)?),
                Err(error) => {
                    let resp = Resp{status: error};
                    Ok(httpcodes::HTTPOk.build().json(resp)?)
                }
            }*/
            Ok(httpcodes::HTTPOk.build().finish()?)
        })
        .responder()
}

fn main() {
//    ::std::env::set_var("RUST_LOG", "actix_web=debug");
    ::std::env::set_var("RUST_LOG", "info,actix_web=debug");
    let _ = env_logger::init();

    let sys = System::new("test");
    let addr = SyncArbiter::start(1, || db::Database::init());

    HttpServer::new(
        move|| Application::with_state(addr.clone() )
            .middleware(middleware::Logger::default())
            .resource("/batch_insert", |r| r.method(Method::POST).f(batch_insert))
            .resource("/search", |r| r.method(Method::POST).f(search))
            .resource("/", |r| r.f(index))
        )
        .bind("0.0.0.0:8080").unwrap()
        .start();

    sys.run();
}