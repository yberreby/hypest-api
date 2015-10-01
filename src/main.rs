#![feature(custom_derive, plugin, io, convert)]
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
extern crate r2d2_postgres;
extern crate crypto; // password hashing
extern crate rand; // for password entropy
extern crate byteorder;

use nickel::{
  Nickel, HttpRouter, StaticFilesHandler, MediaType, QueryString, JsonBody
};

use plugin::{Plugin, Pluggable};
use postgres::SslMode;
use chrono::*;
use r2d2::{NopErrorHandler, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use nickel_postgres::{PostgresMiddleware, PostgresRequestExtensions};
use hyper::header::AccessControlAllowOrigin;
use serde_json::Value;
use byteorder::{BigEndian, ReadBytesExt};

use rustc_serialize::json::{self, Json, ToJson};
use rustc_serialize::base64::ToBase64;
use rustc_serialize::base64;
use rustc_serialize::base64::Config;

use crypto::digest::Digest;
use crypto::bcrypt::bcrypt;

use std::collections::BTreeMap;
use std::io;
use std::io::Cursor;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::mem;


#[derive(Serialize, Deserialize, Debug, RustcDecodable, RustcEncodable)]
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

#[derive(Serialize, Deserialize, Debug, RustcDecodable, RustcEncodable)]
struct PictureMetadata {
    pub author: String,
    pub description: String,
    pub rating: Option<f32>,
    pub gps_lat: f64,
    pub gps_long: f64,
}

#[derive(Serialize, Deserialize, Debug, RustcDecodable, RustcEncodable)]
struct ReturnId {
    pub id: i32,
}

#[derive(Serialize, Deserialize, Debug, RustcDecodable, RustcEncodable)]
struct User {
    // personal data
    pub username: String,
    pub email: String,
    pub password: String,
    // public data
    pub nb_pictures: i32,
    pub hypes: i32,
}

#[derive(Serialize, Deserialize, Debug, RustcDecodable, RustcEncodable)]
struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, RustcDecodable, RustcEncodable)]
struct StatusCode {
    // status code sent back by /login handler,
    // 1 = login correct
    // 0 = login incorrect
    pub code: i32,
}


/// Format the date in the dd/mm/yyyy format.
fn format_date(date: &chrono::NaiveDate) -> String {
    format!("{}/{}/{}", date.day(), date.month(), date.year())
}

/// Returns the base64 of a hash
fn to_base64(input: &[u8]) -> String {
    let config = Config {
        char_set: base64::CharacterSet::Standard,
        newline: base64::Newline::LF,
        pad: false,
        line_length: None
    };

    let hash: String = input.to_base64(config);
    hash
}


fn main() {
    let dbpool = PostgresMiddleware::new("postgresql://postgres:@127.0.0.1/hypest",
                                     SslMode::None,
                                     5,
                                     Box::new(NopErrorHandler)).unwrap();


    let mut server = Nickel::new();
    server.utilize(StaticFilesHandler::new("assets"));
    server.utilize(dbpool);

    server.get("/pictures_in_area", middleware! { |req, mut res| {
        /*
            get all pictures metadatas in the given area
        */

        // HTTP headers
        res.set(MediaType::Json); // Content-Type: application/json
        res.set(AccessControlAllowOrigin::Any); // Disable CORS for AJAX requests

        let conn = req.db_conn();
        let query = req.query();

        // get the show type
        let order_by = query.get("order_by").unwrap();

        // order_by content check
        match order_by {
          "likes" | "rating" | "date_taken" => {},
          _ => panic!("bad input")
        };

        /*
            tl_lat = top left latitude
            tl_long = top left longitude
            br_lat = bottom right latitude
            br_long = bottom right longitude
        */

        // get the border coords
        let tl_lat: f64 = query.get("tl_lat").unwrap().parse().unwrap();
        let tl_long: f64 = query.get("tl_long").unwrap().parse().unwrap();
        let br_lat: f64 = query.get("br_lat").unwrap().parse().unwrap();
        let br_long: f64 = query.get("br_long").unwrap().parse().unwrap();

        let stmt = conn.prepare(&format!("SELECT * FROM pictures
                                 WHERE gps_long BETWEEN SYMMETRIC $1 AND $2
                                 AND gps_lat BETWEEN SYMMETRIC $3 AND $4
                                 AND uploaded=TRUE
                                 ORDER BY {} DESC", order_by)).unwrap();  // prepare the query

        let mut pictures = Vec::new(); // create the PictureDBData vector

        // fill the vector with query's result
        for row in stmt.query(&[&tl_long, &br_long, &tl_lat, &br_lat]).unwrap() {
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

        println!("[*] Instruction exectued");

        serde_json::ser::to_string(&pictures).unwrap() // return the json value of pictures vec
    }});


    // Accepts only JSON
    server.post("/pictures", middleware! { |req, mut res| {
        /*
            inserting picture's metadata into the database.
            the API returns the id of the created row, and returns this id.
            the client then needs to upload the picture.
        */

        res.set(MediaType::Json); // HTTP header : Content-Type: application/json (for return)

        let conn = req.db_conn();
        // retreive the metadata in JSON
        let pic_metadata = req.json_as::<PictureMetadata>().unwrap();

        let stmt = conn.prepare("INSERT INTO pictures
                                (author, description, gps_lat, gps_long, date_taken, rating, uploaded)
                                VALUES($1, $2, $3, $4, NOW(), $5, FALSE)
                                RETURNING id").unwrap();
        let query = stmt.query(&[&pic_metadata.author,
                                &pic_metadata.description,
                                &pic_metadata.gps_lat,
                                &pic_metadata.gps_long,
                                &pic_metadata.rating]);
        let rows = query.iter()
                        .next()
                        .unwrap();

        let first_and_only_row = rows.get(0); // getting the first and only one row
        let pic_id = ReturnId { // creating an ID struct to convert in JSON
            id: first_and_only_row.get("id"),
        };

        serde_json::ser::to_string(&pic_id).unwrap() // returning the id in json
    }});



    server.put("/pictures/:id", middleware! { |req, res| {
        /*
            assuming the iOS client has uploaded the picture,
            this PUT request is for updating "uploaded" column to TRUE
            and uploading the picture's binary
        */

        let conn = req.db_conn();
        let buf_size = 3*1024*1024; // 3mb buffer size

        let pic_id = req.param("id").unwrap()
                                    .parse::<i32>()
                                    .ok()
                                    .expect("invalid id");
        let mut bytes = Vec::<u8>::with_capacity(buf_size); // 3mb buffer size
        req.origin.read_to_end(&mut bytes).unwrap(); // read the request's body

        let mut f = File::create(format!("assets/pictures/{:?}.jpg", pic_id)).unwrap(); // create the file with the given id (in url) as name
        f.write_all(bytes.as_slice()); // write bytes received in the file


        let stmt = conn.prepare("UPDATE pictures
                                SET uploaded=TRUE
                                WHERE id=$1").unwrap(); // update the uploaded column
        stmt.query(&[&pic_id]);

    }});



    server.post("/users", middleware! { |req, mut res| {
        /*
            creates a new user in database
        */

        res.set(MediaType::Json); // HTTP header : Content-Type: application/json (for return)

        let conn = req.db_conn();
        let user_infos = req.json_as::<User>().unwrap();

        // hash the password
        let salt: [u8; 16] = rand::random();
        let cost = 20000;
        let mut password_hash_bin: Vec<u8> = vec![0; 24];

        bcrypt(cost, &salt, &user_infos.password.into_bytes(), &mut password_hash_bin);

        let password_hash = to_base64(&password_hash_bin);

        /*
        let mut buf = Cursor::new(&salt[..]);
        let salt_big_endian = buf.read_u32::<BigEndian>().unwrap();
        */


        let stmt = conn.prepare("INSERT INTO users
                                (username, nick, email, password, date_created, nb_pictures, hypes, salt)
                                VALUES($1, $2, $3, $4, NOW(), $5, $6, $7)
                                RETURNING id").unwrap();

        let salt: &[u8] = &salt;

        let query = stmt.query(&[&user_infos.username,
                    &user_infos.username,
                    &user_infos.email,
                    &password_hash,
                    &user_infos.nb_pictures,
                    &user_infos.hypes,
                    &salt]);


        let rows = query.iter()
                        .next()
                        .unwrap();

        let first_and_only_row = rows.get(0); // getting the first and only one row
        let user_id = ReturnId { // creating an ID struct to convert in JSON
            id: first_and_only_row.get("id"),
        };

        serde_json::ser::to_string(&user_id).unwrap() // returning the id in json

    }});


    server.post("/users/:username", middleware! { |req, mut res| {
        /*
            update an user's infos depending of the content of the JSON body
        */

        /// Update the user's nick with given nick
        fn update_nick(conn: &PooledConnection<PostgresConnectionManager>, username: &String, nick: &Value) {
            let nick_str = nick.as_string().unwrap();
            let stmt = conn.prepare("UPDATE users
                                    SET nick = $1
                                    WHERE username = $2").unwrap();
            let query = stmt.query(&[&nick_str, &username]);
        }

        /// Update the user's email with given email
        fn update_email(conn: &PooledConnection<PostgresConnectionManager>, username: &String, email: &Value) {
            let email_str = email.as_string().unwrap();
            let stmt = conn.prepare("UPDATE users
                                    SET email = $1
                                    WHERE username = $2").unwrap();
            let query = stmt.query(&[&email_str, &username]);
        }

        /// Delete the given user
        fn delete_user(conn: &PooledConnection<PostgresConnectionManager>, username: &String){
            let stmt = conn.prepare("DELETE FROM users
                                    WHERE username = $1").unwrap();
            let query = stmt.query(&[&username]);
        }


        let conn = req.db_conn();

        let username = req.param("username").unwrap().to_owned();

        let mut body = vec![];
        req.origin.read_to_end(&mut body).unwrap();
        let body_utf8 = String::from_utf8(body).unwrap();

        let data: Value = serde_json::from_str(&body_utf8).unwrap();
        let json_body = data.as_object().unwrap();

        for (key, value) in json_body.iter() {
            match &**key { // check what we want to update
                "nick" => update_nick(&conn, &username, value),
                "email" => update_email(&conn, &username, value),
                "delete" => delete_user(&conn, &username),
                _ => {}
            }
        }

    }});

    server.post("/login", middleware! { |req, mut res| {
        res.set(AccessControlAllowOrigin::Any);
        res.set(MediaType::Json); // HTTP header : Content-Type: application/json (for return)

        let conn = req.db_conn();

        let user_infos = req.json_as::<LoginUser>().unwrap();

        // test if email exists
        let stmt = conn.prepare("SELECT email, password, salt
                                FROM users
                                WHERE email = $1
                                LIMIT 1").unwrap();

        let query =  stmt.query(&[&user_infos.email]);

        let rows = query.iter()
                        .next()
                        .unwrap();

        let mut status_code = Vec::new();

        if(rows.len() == 0){
            &status_code.push(StatusCode{
                code: 0
            });

        } else {
            let row = rows.get(0); // getting the row
            let db_email: String = row.get("email");

            if db_email == user_infos.email {
                // now test if password's hash is the same as db's hash
                let db_password: String = row.get("password");
                let db_salt: Vec<u8> = row.get("salt");

                // hash the password with db's salt
                let cost = 20000;
                let mut password_hash_bin: Vec<u8> = vec![0; 24];

                bcrypt(cost, &db_salt, &user_infos.password.into_bytes(), &mut password_hash_bin);

                let password_hash: String = to_base64(&password_hash_bin);

                if db_password == password_hash {
                    // if password matches return a status code 1
                    &status_code.push(StatusCode{
                        code: 1
                    });
                }  else {
                    // return status code 0
                    &status_code.push(StatusCode{
                        code: 0
                    });
                }
            }
        }

        println!("{:?}", serde_json::ser::to_string(&status_code).unwrap());
        serde_json::ser::to_string(&status_code).unwrap()

    }});


    server.listen("0.0.0.0:6767"); // listen

}
