#![cfg_attr(test, allow(dead_code))]

// Web server
extern crate iron;
extern crate persistent;
extern crate router;
extern crate mount;
extern crate staticfile;

extern crate url;
extern crate queryst;

// DB
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

/// Standard lib crates
use std::net::*;
use std::path::Path;

// Iron crates
use iron::prelude::*;
use iron::status;
use iron::mime;

use iron::typemap::Key;
use iron::headers::{self, Headers, ContentType};

use router::Router;
use mount::Mount;
use staticfile::Static;
use persistent::{Read};

// Postgres crates
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager};

// Types

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

pub struct AppDb;
impl Key for AppDb { type Value = PostgresPool; }

// Helper methods
fn setup_connection_pool(connection_str: &str, pool_size: u32) -> PostgresPool {
    let manager = PostgresConnectionManager::new(connection_str, postgres::SslMode::None).unwrap();
    let config = r2d2::Config::builder().pool_size(pool_size).build();
    r2d2::Pool::new(config, manager).unwrap()
}

// Main
fn main() {
    //let pool = setup_connection_pool("postgresql://postgres:@127.0.0.1/hypest", 6);

    let mut router = Router::new();
    router.get("/", move |_: &mut Request| {
      let message = "Hello from a handler".to_owned();
      Ok(Response::with((status::Ok, message)))
    });

    router.get("/pictures_in_area/", move |req: &mut Request| {
      let pool = req.get::<Read<AppDb>>().unwrap();
      let conn = pool.get().unwrap();

      let query_str = &req.url.query.as_ref().expect("missing query string");
      let query = queryst::parse(query_str).unwrap();
      let order_by = query.find("order_by").and_then(|x| x.as_string()).unwrap();

      match order_by {
        "likes" | "rating" | "date_taken" => {},
        _ => panic!("bad input (SQL injection attempt or typo)")
      };


      let left_longitude: f64 = query.find("left_long")
        .and_then(|x| x.as_string())
        .and_then(|x| x.parse())
        .unwrap();

      let right_longitude: f64 = query.find("right_long")
        .and_then(|x| x.as_string())
        .and_then(|x| x.parse())
        .unwrap();
      let top_latitude: f64 = query.find("top_lat").unwrap().parse().unwrap();
      let bottom_latitude: f64 = query.find("bottom_lat").unwrap().parse().unwrap();




      let response_body = "Hello from a handler".to_owned();


      let mut headers = Headers::new();
      headers.set(ContentType::json());

      let mut res = Response::with(response_body);
      res.headers = headers;

      Ok(res)
    });

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/pictures", Static::new(Path::new("./pictures/")));

    let mut middleware = Chain::new(mount);
    //middleware.link(Read::<AppDb>::both(pool));

    let host = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080);
    println!("listening on http://{}", host);
    Iron::new(middleware).http(host).unwrap();
}
