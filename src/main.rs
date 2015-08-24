#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

mod prelude;
use prelude::*;

mod handlers;
mod utils;
use utils::*;

// Main
fn main() {
    let pool = setup_connection_pool("postgresql://postgres:@127.0.0.1/hypest", 6);

    let mut router = Router::new();
    router.get("/", move |_: &mut Request| {
      let message = "Hello from a handler".to_owned();
      Ok(Response::with((status::Ok, message)))
    });

    router.get("/pictures_in_area/", handlers::pictures_in_area_handler);

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/pictures", Static::new(Path::new("./pictures/")));

    let mut middleware = Chain::new(mount);

    // Make the pool available in middlewares.
    middleware.link(Read::<AppDb>::both(pool));

    let host = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080);
    println!("listening on http://{}", host);
    Iron::new(middleware).http(host).unwrap();
}
