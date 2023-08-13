use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use error::AxumHelloError;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};
use tower_http::trace::TraceLayer;

mod config;
mod config_env;
mod error;

#[tokio::main]
async fn main() {
    let config = config::Config::new();
    config::Config::init_otel();
    // let client = config::Config::init_mongo::<User>();

    // Display configured settings
    config.print_values(log::Level::Info);

    let app = Router::new()
        .route("/", get(root))
        .nest("/api", api_router())
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    log::debug!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    tracing::info!("running root");
    "Hello, World!"
}

fn api_router() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

async fn get_user(
    Path(params): Path<HashMap<String, String>>,
) -> Result<Json<User>, AxumHelloError> {
    // insert your application logic here
    match params.get("id") {
        Some(i) => match i.parse::<u64>() {
            Ok(id) => {
                let user = User {
                    id,
                    username: "defaulted".to_string(),
                };
                Ok(Json(user))
            }
            Err(e) => Err(AxumHelloError::BadRequest(format!(
                "failed when parsing id: {}",
                e
            ))),
        },
        None => Err(AxumHelloError::BadRequest(
            "Missing ':id' in path params".to_string(),
        )),
    }
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
