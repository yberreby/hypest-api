use super::prelude::*;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use octavo::digest::sha2::SHA256;
use octavo::digest::Digest;



pub fn check_cookies(req: &mut Request) {
    /*
        check auth cookie: if it's not
        valid, redirect to /login
    */

    fn is_sessid_valid(conn: &PooledConnection<PostgresConnectionManager>, token: &str) {
            /*
                checks if the given sessid exists in database
            */

            // hash the token in sha256
            let mut token_hash_bin: Vec<u8> = vec![0; 32];

            let mut sha2 = SHA256::default();
            sha2.update(token.as_bytes());
            sha2.result(&mut token_hash_bin);

            let token_hash_hex = token_hash_bin.to_hex(); // serialize to hex

            // compare with db's token
            let stmt = conn.prepare("SELECT EXISTS
                                    (SELECT 1 FROM sessions WHERE token_hash = $1 LIMIT 1)
                                    AS exists").unwrap();
            let rows = stmt.query(&[&token_hash_hex]).unwrap();

            let row = rows.get(0); // getting the first and only one row
            let is_session_valid: bool = row.get("exists");

            // TODO: test is_session_valid's value, if it's true,
            // redirect to /map
            // if not,
            // redirect to /login

            // problem: when user goes to /login without session, it creates an infinite loop
            // (login -> no session ? -> redirect to login -> login -> no session ? .... etc)
            // so think about something else

    }


    let conn = req.db_conn();

    if req.origin.headers.has::<Cookie>() {
        let cookie_header = req.origin.headers.get::<Cookie>().unwrap();
        let cookies = &cookie_header.0;

        for cookie in cookies {
            match &*cookie.name {
                "SESSID" => is_sessid_valid(&conn, &cookie.value),
                _ => {}
            }
        }
    }
}
