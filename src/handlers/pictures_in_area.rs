use prelude::*;
use utils::*;

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

#[derive(RustcDecodable, RustcEncodable)]
struct PictureUploadId {
    pub id: i32,
}

pub fn pictures_in_area_handler(req: &mut Request) -> IronResult<Response> {
  let pool = req.get::<Read<AppDb>>().unwrap();
  let conn = pool.get().unwrap();

  let query_str = &req.url.query.as_ref().expect("missing query string");
  let query = queryst::parse(query_str).unwrap();
  let order_by = query.find("order_by").and_then(|x| x.as_string()).unwrap();

  match order_by {
    "likes" | "rating" | "date_taken" => {},
    _ => panic!("bad input (SQL injection attempt or typo)")
  };


  let get_coordinate = |attr_name: &str| -> f64 {
    query.find(attr_name)
     .and_then(|x| x.as_string())
     .and_then(|x| x.parse::<f64>().ok())
     .unwrap()
  };

  let left_longitude = get_coordinate("left_long");
  let right_longitude = get_coordinate("right_long");

  let top_latitude = get_coordinate("top_lat");
  let bottom_latitude = get_coordinate("bottom_lat");




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

  let response_body = String::from_str(serde_json::ser::to_string(&pictures).unwrap());


  let mut headers = Headers::new();
  headers.set(ContentType::json());

  let mut res = Response::with(response_body);
  res.headers = headers;

  Ok(res)
}
