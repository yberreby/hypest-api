#[derive(Serialize, Deserialize, Debug, RustcDecodable, RustcEncodable)]
pub struct PictureDBData {
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
pub struct PictureMetadata {
    pub author: String,
    pub description: String,
    pub rating: Option<f32>,
    pub gps_lat: f64,
    pub gps_long: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReturnId {
    pub id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,
}
