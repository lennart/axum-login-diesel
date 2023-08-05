use axum_login_diesel::axum_login_diesel_store;
use diesel::pg::PgConnection;
use diesel::r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use std::env;
use axum_macros::debug_handler; 
use axum::{
    Extension,
    response::{
        IntoResponse,
    },
    routing::{
        get,
    },
    extract::{State},
};
use axum::Router;
use axum_login::{
    axum_sessions::{
        async_session::MemoryStore,
        SessionLayer,
    },
    AuthLayer, RequireAuthorizationLayer
};
use diesel::prelude::*;
use axum_login_diesel::models::*;
use axum_login_diesel::schema::users::dsl::users;

pub type PostgresUserStore = axum_login_diesel_store::PostgresStore<User>;

type AuthContext = axum_login::extractors::AuthContext<i32, User, PostgresUserStore>;

#[derive(Clone)]
struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
}

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

#[tokio::main]
async fn main() {
    let pool = get_connection_pool();

    let user_store = PostgresUserStore::new(pool.clone());

    let _conn = &mut pool.get().unwrap();

    let secret_string = env::var("SESSION_SECRET").expect("SESSION_SECRET must be set");
    let secret = secret_string.as_bytes();

    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, secret).with_secure(false);

    let auth_layer = AuthLayer::new(user_store, secret);

    #[debug_handler]
    async fn login_handler(State(state): State<AppState>, mut auth: AuthContext) {
        let conn = &mut state.pool.clone().get().unwrap();
        let user: User = users
            .find(1)
            .first(conn)
            .expect("Cannot load user");
        auth.login(&user).await.unwrap();
    }

    async fn logout_handler(State(_state): State<AppState>, mut auth: AuthContext) {
        dbg!("Logging out user: {}", &auth.current_user);
        auth.logout().await;
    }

    async fn protected_handler(State(_state): State<AppState>, Extension(user): Extension<User>) -> impl IntoResponse {
        format!("Logged in as: {}", user.name)
    }

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(RequireAuthorizationLayer::<i32, User>::login())
        .route("/login", get(login_handler))
        .route("/logout", get(logout_handler))
        .layer(auth_layer)
        .layer(session_layer)
        .with_state(AppState { pool: pool.clone() });
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
