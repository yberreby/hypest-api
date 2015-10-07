#![feature(custom_derive, plugin, convert)]
#![plugin(serde_macros)]

#[macro_use]
extern crate nickel; // HTTP server
extern crate hyper;
extern crate postgres; // postgres database management
extern crate chrono; // SQL DATE type management
extern crate nickel_postgres; // postgres middleware
extern crate rustc_serialize; // JSON
extern crate serde; // JSON
extern crate serde_json; // JSON
extern crate r2d2; // pool of threads
extern crate r2d2_postgres;
extern crate octavo;
extern crate rand; // for password entropy
extern crate byteorder;

use nickel::{
  Nickel, HttpRouter, StaticFilesHandler
};

use postgres::SslMode;

use nickel_postgres::{PostgresMiddleware};

use r2d2::NopErrorHandler;

pub use nickel::MediaType;


pub mod db;
mod handlers;

fn main() {
    let dbpool = PostgresMiddleware::new(
      "postgresql://postgres:@127.0.0.1/hypest",
      SslMode::None,
      5,
      Box::new(NopErrorHandler)
    ).unwrap();

    let mut server = Nickel::new();
    server.utilize(StaticFilesHandler::new("assets"));
    server.utilize(dbpool);

    server.get("/pictures_in_area", middleware! { |req, mut res| handlers::pictures_in_area::get(req, &mut res) } );
    server.post("/pictures", middleware! { |req, mut res| handlers::pictures::post(req, &mut res) });
    server.put("/pictures/:id", middleware! { |req, mut res| handlers::pictures::put(req, &mut res) });
    server.post("/users", middleware! { |req, mut res| handlers::users::create_user(req, &mut res) });
    server.post("/users/:username", middleware! { |req, mut res| handlers::users::update_user(req, &mut res) });
    server.post("/login", middleware! { |req, mut res| {
      res.set(MediaType::Json); // HTTP header : Content-Type: application/json (for return)
      
      match handlers::login::post(req, &mut res) {
        Ok(_) => "{\"code\":\"1\"}",
        Err(_) => "{\"code\":\"0\"}"
      }
    }});

    server.listen("0.0.0.0:6767"); // listen
}
