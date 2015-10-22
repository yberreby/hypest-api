use super::prelude::*;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use octavo::digest::sha2::SHA256;
use octavo::digest::Digest;
use rustc_serialize::hex::FromHex;


pub enum SessionStatus {
    Valid,
    Invalid,
}


pub fn check_session(req: &mut Request) -> SessionStatus {
    /*
        check auth cookie: if it's not
        valid, redirect to /login
    */

    fn is_sessid_valid(conn: &PooledConnection<PostgresConnectionManager>, token: &str) -> bool {
            /*
                checks if the given sessid exists in database
            */
            // hash the token in sha256
            let mut token_hash_bin: Vec<u8> = vec![0; 32];
            let token_bin = match token.from_hex(){
                Ok(hex) => hex,
                Err(_) => panic!("invalid sessid"),
            };

            let mut sha2 = SHA256::default();
            sha2.update(token_bin);
            sha2.result(&mut token_hash_bin);

            let token_hash_hex = token_hash_bin.to_hex(); // serialize to hex

            // compare with db's token
            let stmt = conn.prepare("SELECT EXISTS
                                    (SELECT 1 FROM sessions WHERE token_hash = $1 LIMIT 1)
                                    AS exists").unwrap();
            let rows = stmt.query(&[&token_hash_hex]).unwrap();

            let row = rows.get(0); // getting the first and only one row
            let is_session_valid: bool = row.get("exists");

            is_session_valid
            /*
                XXX: continue this when the swift UI is done
            */
    }


    let conn = req.db_conn();

    if req.origin.headers.has::<Cookie>() {
        let cookie_header = req.origin.headers.get::<Cookie>().unwrap();
        let cookies = &cookie_header.0;

        if let Some(session_cookie) = cookies.iter().find(|c| c.name == "SESSID"){
            if is_sessid_valid(&conn, &session_cookie.value) {
                SessionStatus::Valid
            } else {
                SessionStatus::Invalid
            }
        } else {
            SessionStatus::Invalid
        }

        /*
        for cookie in cookies {
            match &*cookie.name {
                "SESSID" => return is_sessid_valid(&conn, &cookie.value),
                _ => return SessionStatus::Invalid
            }
        }
        */
    } else {
        SessionStatus::Invalid
    }
}
