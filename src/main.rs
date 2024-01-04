use axum::{
    extract,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_extra::routing::RouterExt;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Debug, Clone)]
struct AppState {
    hostname: String,
    port: String,
}

impl AppState {
    fn new<S: Into<String>>(hostname: S, port: S) -> Self {
        AppState {
            hostname: hostname.into(),
            port: port.into(),
        }
    }
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState::new(
        env::var("SERVER_HOSTNAME").unwrap_or("localhost".to_string()),
        env::var("SERVER_PORT").unwrap_or("8000".to_string()),
    ));

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route_with_tsr("/handle", get(handle))
        .route_with_tsr("/handle/:name", get(handle))
        .nest_service("/assets", ServeDir::new("assets"))
        .fallback(handler_404)
        .with_state(shared_state.clone());

    // listen
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        {
            if shared_state.hostname != "localhost" {
                println!("{}", shared_state.hostname.clone());
                shared_state.hostname.clone()
            } else {
                "0.0.0.0".to_string()
            }
        },
        shared_state.port
    ))
    .await
    .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    // serve
    axum::serve(listener, app).await.unwrap();
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn root() -> Html<&'static str> {
    Html("<h1>ROOT</h1>")
}

async fn handle(
    extract::State(app_state): extract::State<Arc<AppState>>,
    name: Option<extract::Path<String>>,
) -> Html<String> {
    let name = name.unwrap_or(extract::Path(String::from("INCOGNITO")));

    let mut handlebars = Handlebars::new();
    // let source = "Hello {{ name }}";
    let source = include_str!("../templates/dt-dashboard.html");

    handlebars
        .register_template_string("hello", source)
        .unwrap();

    let mut data = HashMap::new();
    data.insert(
        "url",
        format!("//{}:{}", app_state.hostname, app_state.port),
    );
    data.insert("name", name.to_string());

    println!("{}", data["name"]);

    let rendered = handlebars.render("hello", &data).unwrap();

    // println!("{}", rendered);

    Html(format!("{}", rendered))
}
