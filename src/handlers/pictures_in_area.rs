use super::prelude::*;

/// Format the date in the dd/mm/yyyy format.
fn format_date(date: &chrono::NaiveDate) -> String {
  format!("{}/{}/{}", date.day(), date.month(), date.year())
}

pub fn get(req: &mut Request, res: &mut Response) -> String {
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
      pictures.push(db::PictureDBData {
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
}
