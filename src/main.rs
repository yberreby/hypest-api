#![cfg_attr(test, allow(dead_code))]

extern crate iron;
extern crate persistent;
extern crate router;
extern crate mount;
extern crate staticfile;

extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

/// Standard lib crates
use std::net::*;
use std::path::Path;

// Iron crates
use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
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
fn setup_connection_pool(cn_str: &str, pool_size: u32) -> PostgresPool {
    let manager = PostgresConnectionManager::new(cn_str, ::postgres::SslMode::None).unwrap();
    let config = r2d2::Config::builder().pool_size(pool_size).build();
    r2d2::Pool::new(config, manager).unwrap()
}

// Main
fn main() {
    let pool = setup_connection_pool("postgresql://postgres:@127.0.0.1/hypest", 6);

    let mut router = Router::new();
    router.get("/", move |_: &mut Request| {
      let message = "Hello from a handler".to_owned();
      Ok(Response::with((status::Ok, message)))
    });

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/static", Static::new(Path::new("./src/static/")));

    let mut middleware = Chain::new(mount);
    middleware.link(Read::<AppDb>::both(pool));

    let host = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080);
    println!("listening on http://{}", host);
    Iron::new(middleware).http(host).unwrap();
}
