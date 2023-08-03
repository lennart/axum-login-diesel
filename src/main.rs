use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use axum_login_diesel::models::*;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    use axum_login_diesel::schema::users::dsl::*;
    
    let connection = &mut establish_connection();

    let results = users
        .limit(5)
        .select(User::as_select())
        .load(connection)
        .expect("Uh, oh, cannot load users");

    println!("{} Users", results.len());
    for user in results {
        println!("{}", user.id);
        println!("{}", user.name);
    }
}
