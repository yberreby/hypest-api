use super::prelude::*;
use super::utils;
use rand;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;

// TODO: make sure the email & username doesn't already exist
pub fn create_user(req: &mut Request, res: &mut Response) -> String {
    /*
        user creation handler
    */
    res.set(MediaType::Json); // HTTP header : Content-Type: application/json (for return)

    let conn = req.db_conn();
    let user_data: db::User = serde_json::de::from_reader(&mut req.origin).unwrap();

    // hash the password
    let salt: [u8; 16] = rand::random(); // TODO FIXME XXX An application that requires an entropy source for cryptographic purposes must usr OsRng
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
                &salt]);

    // test if username has already been taken
    match rows {
        Ok(rows) => {
            let first_and_only_row = rows.get(0); // getting the first and only one row
            let user_id = db::ReturnId { // creating an ID struct to convert in JSON
                id: first_and_only_row.get("id"),
            };

            serde_json::ser::to_string(&user_id).unwrap() // returning the id in json
        },

        Err(_) => String::from("{\"code\":\"userame already taken\"}")
    }

}

pub fn update_user(req: &mut Request, _res: &mut Response) {
    /*
        update user handler to update given field
    */
    fn update_nick(conn: &PooledConnection<PostgresConnectionManager>, username: &String, nick: &serde_json::Value) {
        /*
            update user's nick with given nick
        */
        let nick_str = nick.as_string().unwrap();
        let stmt = conn.prepare("UPDATE users
                                SET nick = $1
                                WHERE username = $2").unwrap();
        let _rows = stmt.query(&[&nick_str, &username]).unwrap();
    }

    fn update_email(conn: &PooledConnection<PostgresConnectionManager>, username: &String, email: &serde_json::Value) {
        /*
            update user's email with given email
        */
        let email_str = email.as_string().unwrap();
        let stmt = conn.prepare("UPDATE users
                                SET email = $1
                                WHERE username = $2").unwrap();
        let _rows = stmt.query(&[&email_str, &username]);
    }

    fn update_password(conn: &PooledConnection<PostgresConnectionManager>, username: &String, password: &serde_json::Value) {
        /*
            update the user's password with given password
            TODO: Make sure that the user sends his password
        */
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

    fn delete_user(conn: &PooledConnection<PostgresConnectionManager>, username: &String, password: &serde_json::Value){
        /*
            delete the given user
            TODO: Make sure that the user sends his password
        */



        let stmt = conn.prepare("DELETE FROM users
                                WHERE username = $1").unwrap();
        let _rows = stmt.query(&[&username]);
    }


    let conn = req.db_conn();

    let username = req.param("username").unwrap().to_owned(); // get the username we want to modify

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
            "delete" => delete_user(&conn, &username, value),
            _ => {}
        }
    }

}
