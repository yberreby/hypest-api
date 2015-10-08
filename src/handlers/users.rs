use super::prelude::*;
use super::utils;
use rand;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;

// TODO: make sure the email doesn't already exist
pub fn create_user(req: &mut Request, res: &mut Response) -> String {
    res.set(MediaType::Json); // HTTP header : Content-Type: application/json (for return)

    let conn = req.db_conn();
    let user_data: db::User = serde_json::de::from_reader(&mut req.origin).unwrap();

    // hash the password
    let salt: [u8; 16] = rand::random();
    let salt: &[u8] = &salt;

    let cost = 10;
    let mut password_hash_bin: Vec<u8> = vec![0; 24];

    bcrypt(cost, salt, &user_data.password.into_bytes(), &mut password_hash_bin);

    let password_hash = utils::to_base64(&password_hash_bin);

    let stmt = conn.prepare("INSERT INTO users
                            (username, nick, email, password, date_created, nb_pictures, hypes, salt)
                            VALUES($1, $2, $3, $4, NOW(), 0, 0, $5)
                            RETURNING id").unwrap();

    let rows = stmt.query(&[&user_data.username,
                &user_data.username,
                &user_data.email,
                &password_hash,
                &salt]).unwrap();

    let first_and_only_row = rows.get(0); // getting the first and only one row
    let user_id = db::ReturnId { // creating an ID struct to convert in JSON
        id: first_and_only_row.get("id"),
    };

    serde_json::ser::to_string(&user_id).unwrap() // returning the id in json

}

pub fn update_user(req: &mut Request, _res: &mut Response) {
    /// Update the user's nick with given nick
    fn update_nick(conn: &PooledConnection<PostgresConnectionManager>, username: &String, nick: &serde_json::Value) {
        let nick_str = nick.as_string().unwrap();
        let stmt = conn.prepare("UPDATE users
                                SET nick = $1
                                WHERE username = $2").unwrap();
        let _rows = stmt.query(&[&nick_str, &username]).unwrap();
    }

    /// Update the user's email with given email
    fn update_email(conn: &PooledConnection<PostgresConnectionManager>, username: &String, email: &serde_json::Value) {
        let email_str = email.as_string().unwrap();
        let stmt = conn.prepare("UPDATE users
                                SET email = $1
                                WHERE username = $2").unwrap();
        let _rows = stmt.query(&[&email_str, &username]);
    }

    /// Update the user's password with given password
    // TODO: Make sure that the user sends his old password
    fn update_password(conn: &PooledConnection<PostgresConnectionManager>, username: &String, password: &serde_json::Value) {
        let new_password = password.as_string().unwrap();
        let new_password = String::from(new_password);
        // get the user's salt
        let stmt = conn.prepare("SELECT salt
                                FROM users
                                WHERE username = $1").unwrap();
        let rows = stmt.query(&[&username]).unwrap();

        if rows.len() > 0 {
            let row = rows.get(0);
            let salt: Vec<u8> = row.get("salt");

            // hash
            let cost = 10;
            let mut password_hash_bin: Vec<u8> = vec![0; 24];
            bcrypt(cost, &salt, &new_password.into_bytes(), &mut password_hash_bin);
            let password_hash = utils::to_base64(&password_hash_bin);

            let stmt = conn.prepare("UPDATE users
                                    SET password = $1
                                    WHERE username = $2").unwrap();
            let _rows = stmt.query(&[&password_hash, &username]).unwrap();
        }
    }

    /// Delete the given user
    fn delete_user(conn: &PooledConnection<PostgresConnectionManager>, username: &String){
        let stmt = conn.prepare("DELETE FROM users
                                WHERE username = $1").unwrap();
        let _rows = stmt.query(&[&username]);
    }


    let conn = req.db_conn();

    let username = req.param("username").unwrap().to_owned();

    let mut body = vec![];
    req.origin.read_to_end(&mut body).unwrap();
    let body_utf8 = String::from_utf8(body).unwrap();

    let data: serde_json::Value = serde_json::from_str(&body_utf8).unwrap();
    let json_body = data.as_object().unwrap();

    for (key, value) in json_body.iter() {
        match &**key { // check what we want to update
            "nick" => update_nick(&conn, &username, value),
            "email" => update_email(&conn, &username, value),
            "password" => update_password(&conn, &username, value),
            "delete" => delete_user(&conn, &username),
            _ => {}
        }
    }

}
