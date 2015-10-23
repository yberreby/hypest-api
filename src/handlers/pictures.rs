use super::prelude::*;

// Accepts only JSON

/// Create a new picture in the database, returning its ID.
///
/// After calling this endpoint, the client will need to upload
/// the picture using the returned ID.
pub fn post(req: &mut Request, res: &mut Response) -> String {
    res.set(MediaType::Json);

    let conn = req.db_conn();
    let pic_metadata: db::PictureMetadata = {
        serde_json::de::from_reader(&mut req.origin).unwrap()
    };


    let stmt = conn.prepare("INSERT INTO pictures
                            (
                                author,
                                description,
                                gps_lat,
                                gps_long,
                                date_taken,
                                rating,
                                uploaded
                            )
                            VALUES($1, $2, $3, $4, NOW(), $5, FALSE)
                            RETURNING id").unwrap();
    let rows = stmt.query(&[&pic_metadata.author,
                            &pic_metadata.description,
                            &pic_metadata.gps_lat,
                            &pic_metadata.gps_long,
                            &pic_metadata.rating]).unwrap();

    let first_and_only_row = rows.get(0);
    let pic_id = db::ReturnId { // creating an ID struct to convert in JSON
        id: first_and_only_row.get("id"),
    };

    serde_json::ser::to_string(&pic_id).unwrap()
}


/// Upload a picture with the given ID, after it has been created in the DB.
///
/// This will mark the picture as uploaded once the transfer is complete.
pub fn put(req: &mut Request, _res: &mut Response) {
    let conn = req.db_conn();
    let buf_size = 3*1024*1024; // 3mb buffer size

    let pic_id = req.param("id").unwrap()
                                .parse::<i32>()
                                .ok()
                                .expect("invalid id");
    let mut bytes = Vec::<u8>::with_capacity(buf_size); // 3mb buffer size
    req.origin.read_to_end(&mut bytes).unwrap(); // read the request's body

    let mut f = File::create(
        format!("assets/pictures/{:?}.jpg", pic_id)
    ).unwrap(); // create the file with the given id (in url) as name
    f.write_all(bytes.as_slice()).unwrap();


    let stmt = conn.prepare("UPDATE pictures
                            SET uploaded=TRUE
                            WHERE id=$1").unwrap();
    stmt.query(&[&pic_id]).unwrap();
}

