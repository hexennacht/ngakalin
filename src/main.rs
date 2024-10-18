use axum::http::{Response, StatusCode};
use axum::{Router};
use axum::extract::{Request, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put, MethodRouter};
use bb8::{Pool};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use structopt::StructOpt;
use crate::endpoint::{Service, Source};

mod endpoint;
mod config;
mod cmd;

#[tokio::main]
async fn main() {
    blocking().await;
    tracing_subscriber::fmt::init();

    let conf = cmd::Command::from_args();

    let configuration =  config::Configuration::read_config_file(conf.config)
        .await
        .unwrap();

    let endpoint_config = endpoint::Configuration::read_config(configuration.clone().response.config_file)
        .await
        .unwrap();

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", configuration.clone().service.host, configuration.clone().service.port)
    ).await.unwrap();

    let services = endpoint_config.clone().services;

    let route = register_route(services).await;

    tracing::info!("Server started");
    axum::serve(listener, route.with_state(AppState{redis_connection: connect_redis().await})).await.unwrap()
}

async fn register_route(services: Vec<Service>) -> Router<AppState> {
    let mut app: Router<AppState> = Router::new();
    let mut routers: Vec<(String, MethodRouter<AppState>)> = vec![];

    services.into_iter()
        .for_each(|service| routers.append(&mut create_route(service)));

    routers.into_iter()
        .for_each(|route| app = app.clone().route(route.clone().0.as_str(), route.clone().1.clone()));

    app
}

async fn connect_redis() -> Pool<RedisConnectionManager> {
    tracing::info!("connecting to redis");

    let manager = RedisConnectionManager::new("redis://127.0.0.1:6379").unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();
    {
        // ping the database before starting
        let mut conn = pool.get().await.unwrap();
        conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
        let result: String = conn.get("foo").await.unwrap();
        assert_eq!(result, "bar");
    }

    tracing::info!("successfully connected to redis and pinged it");

    pool
}

fn create_route(service: Service) -> Vec<(String, MethodRouter<AppState>)> {
    let sources = service.clone().sources;

    sources.into_iter()
        .map(|source| {
            let route = match source.clone().method.as_str() {
                "GET" => get(handle_mock_response),
                "POST" => post(handle_mock_response),
                "PUT" => put(handle_mock_response),
                "DELETE" => delete(handle_mock_response),
                "PATCH" => patch(handle_mock_response),
                _ => unreachable!()
            };

            let endpoint = format!("{}{}", service.clone().prefix, source.clone().endpoint);

            (endpoint, route)
        }).collect()
}

type ConnectionPool = Pool<RedisConnectionManager>;

#[derive(Clone)]
pub struct AppState {
    pub redis_connection: ConnectionPool
}

async fn handle_mock_response(
    State(state): State<AppState>,
    req: Request
) -> impl IntoResponse {
    let redis_connection = state.clone().redis_connection.clone();
    tracing::info!("mock response: {:?} {:?}", req.uri().clone().host(), req.uri().clone().path());
    let mut conn = redis_connection.get().await.unwrap();

    let res: String = conn.get("foo").await.unwrap();

    tracing::info!("response: {}", res.clone());

    let endpoint_config: Source = serde_json::from_str::<Source>(res.as_str()).unwrap();

    let status_code = StatusCode::from_u16(endpoint_config.clone().status).unwrap();

    let body: String = match endpoint_config.clone().content_type.as_str() {
        "application/json" => {
            std::fs::read_to_string(&endpoint_config.response).unwrap()
        }
        "text/html" | "application/xml" | "text/plain" => {
            std::fs::read_to_string(&endpoint_config.response).unwrap()
        }
        _ => {
            "Unsupported content type".to_string()
        }
    };

    Response::builder()
        .status(status_code)
        .header(axum::http::header::CONTENT_TYPE, &endpoint_config.content_type)
        .body(body)
        .unwrap()
}

async fn blocking() {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    coba(sender).await;

    println!("Hello, world!");
    println!("{}", receiver.await.unwrap());
}

async fn coba(channel: tokio::sync::oneshot::Sender<String>) {
    channel
        .send("Hello, world from channel".to_string())
        .unwrap()
}