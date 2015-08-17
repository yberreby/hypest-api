#![cfg_attr(test, allow(dead_code))]

extern crate iron;
extern crate persistent;
extern crate router;
extern crate mount;
extern crate staticfile;

extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

extern crate rustc_serialize;

/// Standard lib crates
use std::env;
use std::net::*;
use std::path::Path;
use std::collections::BTreeMap;

// Json crates
use rustc_serialize::json;
use rustc_serialize::json::{ToJson, Json};

// Iron crates
use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use router::Router;
use mount::Mount;
use staticfile::Static;
use persistent::{Write,Read};

// Postgres crates
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager};

// Types

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

#[derive(Copy, Clone)]
pub struct HitCounter;
impl Key for HitCounter { type Value = usize; }

pub struct AppDb;
impl Key for AppDb { type Value = PostgresPool; }

struct Team {
    name: String,
    points: u16
}

impl ToJson for Team {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("name".to_string(), self.name.to_json());
        m.insert("points".to_string(), self.points.to_json());
        m.to_json()
    }
}

// Helper methods
fn setup_connection_pool(cn_str: &str, pool_size: u32) -> PostgresPool {
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(cn_str, ::postgres::SslMode::None).unwrap();
    let config = ::r2d2::Config::builder().pool_size(pool_size).build();
    ::r2d2::Pool::new(config, manager).unwrap()
}

fn insert_dummy_data(conn :&PostgresPooledConnection) {
    conn.execute("DROP TABLE IF EXISTS messages;", &[]).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS messages (id INT PRIMARY KEY);", &[]).unwrap();
    conn.execute("INSERT INTO messages VALUES (1);", &[]).unwrap();
    conn.execute("INSERT INTO messages VALUES (2);", &[]).unwrap();
    conn.execute("INSERT INTO messages VALUES (3);", &[]).unwrap();
}

fn get_team_dummy_data() -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();
    let teams = vec![
        Team { name: "Jake Scott".to_string(), points: 11u16 },
        Team { name: "Adam Reeve".to_string(), points: 19u16 },
        Team { name: "Richard Downer".to_string(), points: 22u16 }
    ];
    data.insert("teams".to_string(), teams.to_json());
    data
}

// Routes
fn environment(_: &mut Request) -> IronResult<Response> {
    let powered_by:String = match env::var("POWERED_BY") {
        Ok(val) => val,
        Err(_) => "Iron".to_string()
    };
    let message = format!("Powered by: {}, pretty cool aye", powered_by);
    Ok(Response::with((status::Ok, message)))
}

fn json(_: &mut Request) -> IronResult<Response> {
    let data = get_team_dummy_data();
    let encoded = json::encode(&data).unwrap();
    let mut response = Response::new();
    response.set_mut(status::Ok);
    response.set_mut(encoded);
    Ok(response)
}

fn posts(req: &mut Request) -> IronResult<Response> {
    let ref post_id = req.extensions.get::<Router>().unwrap().find("post_id").unwrap_or("none");
    Ok(Response::with((status::Ok, "PostId: {}", *post_id)))
}

fn hits(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<HitCounter>>().unwrap();
    let mut count = mutex.lock().unwrap();
    *count += 1;
    Ok(Response::with((status::Ok, format!("Hits: {}", *count))))
}

fn database(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();
    let stmt = conn.prepare("SELECT id FROM messages;").unwrap();
    for row in stmt.query(&[]).unwrap() {
        let id: i32 = row.get(0);
        println!("id: {}", id);
    }
    Ok(Response::with((status::Ok, format!("Db: {}", "ok"))))
}

// Main
fn main() {
    let conn_string:String = match env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(_) => "postgres://dbuser:dbpass@localhost:5432/test".to_string()
    };

    println!("connecting to postgres: {}", conn_string);
    let pool = setup_connection_pool(&conn_string, 6);
    let conn = pool.get().unwrap();

    println!("inserting dummy data.");
    insert_dummy_data(&conn);

    let mut router = Router::new();
    router.get("/", environment);
    router.get("/json", json);
    router.get("/posts/:post_id", posts);
    router.get("/hits", hits);
    router.get("/database", database);

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/static", Static::new(Path::new("./src/static/")));

    let mut middleware = Chain::new(mount);
    middleware.link(Write::<HitCounter>::both(0));
    middleware.link(Read::<AppDb>::both(pool));

    let host = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080);
    println!("listening on http://{}", host);
    Iron::new(middleware).http(host).unwrap();
}
