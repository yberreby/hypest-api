#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate nickel; // HTTP server
extern crate postgres; // postgres database management
extern crate chrono; // SQL DATE type management
extern crate nickel_postgres; // postgres middleware

extern crate serde; // JSON
extern crate serde_json; // JSON
extern crate r2d2; // pool of threads

use nickel::{
  Nickel, HttpRouter, StaticFilesHandler, MediaType, QueryString
};
use postgres::SslMode;
use chrono::*;
use r2d2::NopErrorHandler;
use nickel_postgres::PostgresMiddleware;
use nickel_postgres::PostgresRequestExtensions;


#[derive(Serialize, Deserialize, Debug)]
struct PictureMetadata {
    pub id: i32,
    pub author: String,
    pub description: String,
    pub gps_lat: f64,
    pub gps_long: f64,
    pub date_taken: String,
    pub rating: Option<f32>,
    pub likes: i32,
}


/// Format the date in the dd/mm/yyyy format.
fn format_date(date: &chrono::NaiveDate) -> String {
    format!("{}/{}/{}", date.day(), date.month(), date.year())
}


fn main() {
    let dbpool = PostgresMiddleware::new("postgresql://postgres:@127.0.0.1/hypest",
                                     SslMode::None,
                                     5, // <--- number of connections to the DB, I think
                                     Box::new(NopErrorHandler)).unwrap();


    let mut server = Nickel::new();
    server.utilize(StaticFilesHandler::new("/home/jhun/Code/rust/assets/"));
    server.utilize(dbpool);

    server.get("/pictures_in_area/", middleware! { |req, mut res| {
        /*
            get all pictures in the given area
            LATITUDE: y (lat -> colone -> vertical)
            LONGITUDE: x (long -> ligne -> horizontal)
        */

        res.set(MediaType::Json); // HTTP header : Content-Type: application/json

        let conn = req.db_conn();
        let query = req.query();

        // get the show type
        let order_by = query.get("order_by").unwrap();

        // On vérifie juste que order_by ait une valeur connue, si ce n'est pas le cas, panic.
        match order_by {
          "likes" | "rating" | "date_taken" => {},
          _ => panic!("bad input (SQL injection attempt or typo)")
        };

        // get the border coords
        let left_longitude: f64 = query.get("left_long").unwrap().parse().unwrap();
        let right_longitude: f64 = query.get("right_long").unwrap().parse().unwrap();
        let top_latitude: f64 = query.get("top_lat").unwrap().parse().unwrap();
        let bottom_latitude: f64 = query.get("bottom_lat").unwrap().parse().unwrap();

        let stmt = conn.prepare(&format!("SELECT * FROM pictures
                                 WHERE gps_long BETWEEN SYMMETRIC $1 AND $2
                                 AND gps_lat BETWEEN SYMMETRIC $3 AND $4
                                 AND uploaded=TRUE
                                 ORDER BY {} DESC LIMIT 50", order_by)).unwrap();  // prepare the query

        let mut pictures = Vec::new(); // create the PictureMetadata vector

        // fill the vector with query's result
        for row in stmt.query(&[&left_longitude, &right_longitude, &top_latitude, &bottom_latitude]).unwrap() {
            pictures.push(PictureMetadata {
                id: row.get("id"),
                author: row.get("author"),
                description: row.get("description"), // optional
                gps_lat: row.get("gps_lat"),
                gps_long: row.get("gps_long"),
                date_taken: format_date(&row.get("date_taken")),
                rating: row.get("rating"), // optional
                likes: row.get("likes"),
            });
        }

        serde_json::ser::to_string(&pictures).unwrap() // return the json value of pictures vec
    }});



    server.post("/pictures/", middleware! { |req, res| {
        let conn = req.db_conn();
        let query = req.query();

        let prenom = query.get("pseudo").unwrap();
    }});

    /*
    server.get("/pictures/:id", middleware! { |req, mut res| {
        let conn = req.db_conn();

        res.set(MediaType::Json); // HTTP header : Content-Type: application/json

        let photo_id: i32 = req.param("id").unwrap().parse().unwrap(); // get the :id param
        println!("Photo {} request", photo_id); // debug photo request


        let stmt = conn.prepare("SELECT * FROM pictures WHERE id=$1 AND uploaded=TRUE").unwrap(); // prepare the query
        let query = stmt.query(&[&photo_id]).unwrap(); // execute the query with photo_id
        let row = query.iter().next().unwrap(); // get the query's result

        let date: chrono::NaiveDate = row.get("date_taken");
        let date = format!("{}/{}/{}", date.day(), date.month(), date.year());

        // fill PictureMetadata with data
        let data = PictureMetadata {
            id: row.get("id"),
            author: row.get("author"),
            description: row.get("description"), // optional
            gps_lat: row.get("gps_lat"),
            gps_long: row.get("gps_long"),
            date_taken: date,
            rating: row.get("rating"),
            likes: row.get("likes"),
        };

        serde_json::ser::to_string(&data).unwrap() // return the json of data (PictureMetadata)
    }});
    */


    server.listen("127.0.0.1:6767"); // listen

}
