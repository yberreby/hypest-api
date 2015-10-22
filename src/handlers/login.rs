use super::prelude::*;
use super::utils;

use std::cell::RefCell;
use rand::os::OsRng;
use rand::{Rand, Rng};

use octavo::digest::sha2::SHA256;
use octavo::digest::Digest;

#[derive(Serialize, Deserialize, Debug)]
struct UserCredentials {
    pub email: String,
    pub password: String,
}

pub enum LoginStatus {
    LoginOk,
    EmailIncorrect,
    PasswordIncorrect,
}


thread_local!(static OS_RNG: RefCell<OsRng> = RefCell::new(OsRng::new().unwrap()));

fn os_random<T: Rand>() -> T {
    OS_RNG.with(|r| {
        r.borrow_mut().gen()
    })
}


// TODO: use a proper status enum to represent the different failure modes
// - missing email
// - incorrect password
pub fn post(req: &mut Request, res: &mut Response) -> LoginStatus {
    /*
        login with email and password
    */
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
        return LoginStatus::EmailIncorrect; // email doesn't exists
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

                // generate the token
                let token: [u8; 32] = os_random();
                let token: &[u8] = &token;

                let mut token_hash_bin: Vec<u8> = vec![0; 32];

                let mut sha2 = SHA256::default();
                sha2.update(token); // hash the token
                sha2.result(&mut token_hash_bin);

                // STORE THIS HASHED TOKEN HEX TO DATABASE
                let token_hash_hex = token_hash_bin.to_hex(); // serialize the token hash to hex

                // RETURN THIS UNHASHED TOKEN HEX IN SET-COOKIE
                let token_hex = token.to_hex(); // serialize the token to hex
                println!("{}", token_hex);

                // create session row in database
                let stmt = conn.prepare("INSERT INTO sessions
                                        (username, token_hash, date_created)
                                        VALUES($1, $2, NOW())").unwrap();
                let _query = stmt.query(&[&username, &token_hash_hex]).unwrap();

                return LoginStatus::LoginOk;
            }  else {
                return LoginStatus::PasswordIncorrect;
            }
        } else {
            return LoginStatus::EmailIncorrect;
        }
    }
}
