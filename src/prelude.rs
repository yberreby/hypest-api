// Web server
extern crate iron;
extern crate persistent;
extern crate router;
extern crate mount;
extern crate staticfile;

extern crate url;
extern crate queryst as queryst_;
pub use self::queryst_ as queryst;

// DB
extern crate postgres as postgres_;
pub use self::postgres_ as postgres;
extern crate r2d2 as r2d2_;
pub use self::r2d2_ as r2d2;
extern crate r2d2_postgres;

extern crate serde as serde_;
pub use self::serde_ as serde;
extern crate serde_json as serde_json_;
pub use self::serde_json_ as serde_json;

/// Standard lib crates
pub use std::net::*;
pub use std::path::Path;

// Iron crates
pub use self::iron::prelude::*;
pub use self::iron::status;
pub use self::iron::mime;

pub use self::iron::typemap::Key;
pub use self::iron::headers::{self, Headers, ContentType};

pub use self::router::Router;
pub use self::mount::Mount;
pub use self::staticfile::Static;
pub use self::persistent::{Read};

// Postgres crates
pub use self::r2d2::{Pool, PooledConnection};
pub use self::r2d2_postgres::{PostgresConnectionManager};
