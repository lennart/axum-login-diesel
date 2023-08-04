use axum_login_diesel::axum_login_diesel_store;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use axum_login_diesel::models::*;

pub type PostgresUserStore = axum_login_diesel_store::PostgresStore<<axum_login_diesel::schema::users::table as diesel::associations::HasTable>::Table, axum_login_diesel::schema::users::dsl::id, User>;


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let url =     env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(url);

    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}


fn main() {
    use axum_login_diesel::schema::users::dsl::*;
    let pool = get_connection_pool();
;
    let user_store = PostgresUserStore::new(pool.clone());

    let conn = &mut pool.get().unwrap();
    
    let results = users
        .limit(5)
        .select(User::as_select())
        .load(conn)
        .expect("Uh, oh, cannot load users");

    println!("{} Users", results.len());
    for user in results {
        println!("{}", user.id);
        println!("{}", user.name);
    }
}
