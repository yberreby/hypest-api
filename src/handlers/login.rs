use super::prelude::*;
use super::utils;
use rand;

#[derive(Serialize, Deserialize, Debug)]
struct UserCredentials {
    pub email: String,
    pub password: String,
}

// TODO: use a proper status enum to represent the different failure modes
// - missing email
// - incorrect password
/// login function with email and password
pub fn post(req: &mut Request, res: &mut Response) -> Result<(), ()> {
    res.set(AccessControlAllowOrigin::Any);

    let conn = req.db_conn();

    let credentials: UserCredentials = serde_json::de::from_reader(&mut req.origin).unwrap();

    // test if email exists
    let stmt = conn.prepare("SELECT username, email, password, salt
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
                // session creation processus
                let username: String = row.get("username");

                // salt generation
                let salt: [u8; 16] = rand::random(); // TODO FIXME XXX An application that requires an entropy source for cryptographic purposes must usr OsRng
                let salt: &[u8] = &salt;

                // generate the token
                let token: [u8; 32] = rand::random(); // TODO FIXME XXX An application that requires an entropy source for cryptographic purposes must usr OsRng
                let token: &[u8] = &token;

                let cost = 3;
                let mut token_hash_bin: Vec<u8> = vec![0; 24];

                bcrypt(cost, salt, &token, &mut token_hash_bin); // hash it in database only
                let token_hash_hex = token_hash_bin.to_hex(); // serialize to hex

                // create session row in database
                let stmt = conn.prepare("INSERT INTO sessions
                                        (username, token_hash, salt, date_created)
                                        VALUES($1, $2, $3, NOW()").unwrap();
                let _query = stmt.query(&[&username,
                                        &token_hash_hex,
                                        &salt]).unwrap();

                return Ok(());
            }  else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }
}
