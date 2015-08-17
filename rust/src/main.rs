#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate nickel; // HTTP server
extern crate hyper;
extern crate postgres; // postgres database management
extern crate chrono; // SQL DATE type management
extern crate nickel_postgres; // postgres middleware
extern crate plugin;
extern crate rustc_serialize; // JSON
extern crate serde; // JSON
extern crate serde_json; // JSON
extern crate r2d2; // pool of threads

use nickel::{
  Nickel, HttpRouter, StaticFilesHandler, MediaType, QueryString, JsonBody
};
use plugin::{Plugin, Pluggable};
use postgres::SslMode;
use chrono::*;
use r2d2::NopErrorHandler;
use nickel_postgres::PostgresMiddleware;
use nickel_postgres::PostgresRequestExtensions;
use rustc_serialize::json::{self, Json, ToJson};
use std::collections::BTreeMap;



#[derive(Serialize, Deserialize, Debug)]
struct PictureDBData {
    pub id: i32,
    pub author: String,
    pub description: String,
    pub gps_lat: f64,
    pub gps_long: f64,
    pub date_taken: String,
    pub rating: Option<f32>, // reting is set to -1 when there's no rating.
    pub likes: i32, // likes as 0 value default
}

#[derive(RustcDecodable, RustcEncodable)]
struct PictureMetadata {
    pub author: String,
    pub description: String,
    pub rating: Option<f32>,
    pub gps_lat: f64,
    pub gps_long: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct PictureReturnId {
    pub id: i32,
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

        let mut pictures = Vec::new(); // create the PictureDBData vector

        // fill the vector with query's result
        for row in stmt.query(&[&left_longitude, &right_longitude, &top_latitude, &bottom_latitude]).unwrap() {
            pictures.push(PictureDBData {
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


    // Accepts only JSON
    server.post("/pictures/", middleware! { |req, mut res| {
        let conn = req.db_conn();
        res.set(MediaType::Json); // HTTP header : Content-Type: application/json

        // retreive the metadata in JSON
        let pic_metadata = req.json_as::<PictureMetadata>().unwrap();

        let stmt = conn.prepare("INSERT INTO pictures (author, description, gps_lat, gps_long, date_taken, rating, uploaded)
                                VALUES($1, $2, $3, $4, NOW(), -1, FALSE)
                                RETURNING id").unwrap();

        let query = stmt.query(&[&pic_metadata.author, &pic_metadata.description, &pic_metadata.gps_lat, &pic_metadata.gps_long]);
        let rows = query.iter()
                        .next()
                        .unwrap();

        let first_and_only_row = rows.get(0); // getting the first and only one row

        let pic_id = PictureReturnId { // creating an ID struct to convert in JSON
            id: first_and_only_row.get("id"),
        };

        serde_json::ser::to_string(&pic_id).unwrap() // returning the id in json
    }});


    
    server.put("/pictures/:id", middleware! { |req, res| {
            // WIP
    }});


    server.listen("127.0.0.1:6767"); // listen

}
