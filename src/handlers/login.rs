use super::prelude::*;
use super::utils;

#[derive(Serialize, Deserialize, Debug)]
struct UserCredentials {
    pub email: String,
    pub password: String,
}

// TODO: use a proper status enum to represent the different failure modes
// - missing email
// - incorrect password

pub fn post(req: &mut Request, res: &mut Response) -> Result<(), ()> {
    res.set(AccessControlAllowOrigin::Any);

    let conn = req.db_conn();

    let credentials: UserCredentials = serde_json::de::from_reader(&mut req.origin).unwrap();

    // test if email exists
    let stmt = conn.prepare("SELECT email, password, salt
                            FROM users
                            WHERE email = $1
                            LIMIT 1").unwrap();

    let rows = stmt.query(&[&credentials.email]).unwrap();

    if rows.len() == 0 {
        return Err(());
    } else {
        let row = rows.get(0); // getting the row
        let db_email: String = row.get("email");

        if db_email == credentials.email {
            // now test if password's hash is the same as db's hash
            let db_password: String = row.get("password");
            let db_salt: Vec<u8> = row.get("salt");

            // hash the password with db's salt
            let cost = 10;
            let mut password_hash_bin: Vec<u8> = vec![0; 24];

            bcrypt(cost, &db_salt, &credentials.password.into_bytes(), &mut password_hash_bin);

            let password_hash: String = utils::to_base64(&password_hash_bin);

            if db_password == password_hash {
              return Ok(());
            }  else {
              return Err(());
            }
        } else {
          return Err(());
        }
    }
}
