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
use std::io;


mod db;

#[derive(Serialize, Debug)]
pub struct Resp {
    status: &'static str
}

#[derive(Serialize, Debug)]
pub struct RespWithMessage {
    status: &'static str,
    message: &'static str
}

impl Actor for db::Database {
    type Context = SyncContext<Self>;
}

impl Message for db::TicketsInsertRequest {
    type Result = Result<bool, io::Error>;
}

impl Message for db::SearchRequest {
    type Result = Result<db::Solutions, RespWithMessage>;
}

impl Handler<db::TicketsInsertRequest> for db::Database {
    type Result = Result<bool, io::Error>;

    fn handle(&mut self, msg: db::TicketsInsertRequest, _ctx: &mut Self::Context) -> Self::Result {
        self.insert_tickets(msg.tickets);
        Ok(true)
    }
}

impl Handler<db::SearchRequest> for db::Database {
    type Result = Result<db::Solutions, RespWithMessage>;//??

    fn handle(&mut self, msg: db::SearchRequest, _ctx: &mut Self::Context) -> Self::Result {
        match self.search_flights(&msg) {
            Ok(solutions) => Ok(solutions),
            Err(error) => {
                Err(RespWithMessage { status: "failure", message: error })
            }
        }
    }
}

fn index(_req: HttpRequest<actix::Addr<actix::Syn, db::Database>>) -> &'static str {
    "Hello world"
}

fn batch_insert(req: HttpRequest<actix::Addr<actix::Syn, db::Database>>) -> Box<Future<Item=HttpResponse, Error=Error>> {

    let db_actor: actix::Addr<actix::Syn, db::Database> = req.state().clone();

    req.concat2()
        .then(move|json_res|{
            match json_res {
                Ok(json_body) => {
                    let unser_res = serde_json::from_slice::<db::TicketsInsertRequest>(&json_body);
                    match unser_res.map(|insert_req|{
                        db_actor.send(insert_req)
                    }){
                        Ok(send_res) => Some(send_res),
                        Err(e) => {
                            error!("Couldn't parse json with following error: {}", e.to_string());
                            None
                        }
                    }

                },
                Err(_e) => {
                    error!("Payload error");
                    None
                }
            }

        }).then(|res| {
            match res.map(|res_opt|{
                match res_opt {
                    Some(res) =>  if res.is_ok()  { Resp { status: "success" }} else { Resp { status: "failure" } },
                    None => Resp { status: "failure" }
                }
            }) {
                Ok(resp) => Ok(httpcodes::HttpOk.build().json(resp)?),
                Err(_e) => Ok(httpcodes::HttpOk.build().json(Resp { status: "failure" })?)
            }
        }).responder()

}

fn search(req: HttpRequest<actix::Addr<actix::Syn, db::Database>>) ->Box<Future<Item=HttpResponse, Error=Error>> {

    let db_actor: actix::Addr<actix::Syn, db::Database> = req.state().clone();

    req.concat2()
        .then(move|json_res|{
            match json_res {
                Ok(json_body) => {
                    let unser_res = serde_json::from_slice::<db::SearchRequest>(&json_body);
                    match unser_res.map(|search_req|{
                        db_actor.send(search_req)
                    }){
                        Ok(send_res) => Some(send_res),
                        Err(e) => {
                            error!("Couldn't parse json with following error: {}", e.to_string());
                            None
                        }
                    }

                },
                Err(_e) => {
                    error!("Payload error");
                    None
                }
            }

        }).then(|res| {
            match res.map(|res_opt|{
                match res_opt {
                    Some(res) => res,
                    None => Err(RespWithMessage { status: "failure", message: ""})
                }
            }).map(|resp|{
                if resp.is_ok() {
                    Ok(httpcodes::HttpOk.build().json(resp.unwrap())?)
                } else {
                    Ok(httpcodes::HttpOk.build().json(resp.unwrap_err())?)
                }
            }){
                Ok(t) => t,
                Err(_e) => Ok(httpcodes::HttpOk.build().json(Resp { status: "failure"})?)
            }
        }).responder()
}

fn main() {

//    ::std::env::set_var("RUST_LOG", "info");
    ::std::env::set_var("RUST_LOG", "error");
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