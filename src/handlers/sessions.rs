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


/// Check if the given session ID is linked to a valid session.
fn is_sessid_valid(
    conn: &PooledConnection<PostgresConnectionManager>,
    token: &str
) -> bool {
        // Hash the token with SHA-256.
        // It's okay to use SHA-256 here, because session tokens have
        // much more entropy than human-generated passwords.
        let mut token_hash_bin: Vec<u8> = vec![0; 32];
        let token_bin = match token.from_hex(){
            Ok(hex) => hex,
            Err(_) => panic!("session ID was not valid hex"),
        };

        let mut sha2 = SHA256::default();
        sha2.update(token_bin);
        sha2.result(&mut token_hash_bin);

        let token_hash_hex = token_hash_bin.to_hex();

        // TODO: audit this code.
        // compare with db's token
        let stmt = conn.prepare("SELECT EXISTS
                                (
                                    SELECT 1 FROM sessions
                                    WHERE token_hash = $1
                                    LIMIT 1
                                )
                                AS exists").unwrap();
        let rows = stmt.query(&[&token_hash_hex]).unwrap();

        let row = rows.get(0); // getting the first and only one row
        let is_session_valid: bool = row.get("exists");

        is_session_valid
        /*
            XXX: continue this when the swift UI is done
        */
}


/// Check the validity of a request's session cookie.
pub fn check_session(req: &mut Request) -> SessionStatus {
    let conn = req.db_conn();

    if req.origin.headers.has::<Cookie>() {
        let cookie_header = req.origin.headers.get::<Cookie>().unwrap();
        let cookies = &cookie_header.0;

        if let Some(session_cookie) = cookies.iter().find(|c| {
            c.name == "SESSID"
        }) {
            if is_sessid_valid(&conn, &session_cookie.value) {
                SessionStatus::Valid
            } else {
                SessionStatus::Invalid
            }
        } else {
            SessionStatus::Invalid
        }
    } else {
        // No session cookie was found
        SessionStatus::Invalid
    }
}

