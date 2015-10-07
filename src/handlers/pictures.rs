use super::prelude::*;

// Accepts only JSON
pub fn post(req: &mut Request, res: &mut Response) -> String {
    /*
        inserting picture's metadata into the database.
        the API returns the id of the created row, and returns this id.
        the client then needs to upload the picture.
    */

    res.set(MediaType::Json); // HTTP header : Content-Type: application/json (for return)

    let conn = req.db_conn();
    // retreive the metadata in JSON
    let pic_metadata: db::PictureMetadata = serde_json::de::from_reader(&mut req.origin).unwrap();


    let stmt = conn.prepare("INSERT INTO pictures
                            (author, description, gps_lat, gps_long, date_taken, rating, uploaded)
                            VALUES($1, $2, $3, $4, NOW(), $5, FALSE)
                            RETURNING id").unwrap();
    let rows = stmt.query(&[&pic_metadata.author,
                            &pic_metadata.description,
                            &pic_metadata.gps_lat,
                            &pic_metadata.gps_long,
                            &pic_metadata.rating]).unwrap();

    let first_and_only_row = rows.get(0); // getting the first and only one row
    let pic_id = db::ReturnId { // creating an ID struct to convert in JSON
        id: first_and_only_row.get("id"),
    };

    serde_json::ser::to_string(&pic_id).unwrap() // returning the id in json
}

pub fn put(req: &mut Request, _res: &mut Response) {
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
    f.write_all(bytes.as_slice()).unwrap(); // write bytes received in the file


    let stmt = conn.prepare("UPDATE pictures
                            SET uploaded=TRUE
                            WHERE id=$1").unwrap(); // update the uploaded column
    stmt.query(&[&pic_id]).unwrap();
}
