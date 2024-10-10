use std::sync::Arc;
use axum::http::{Response, StatusCode};
use axum::{Router};
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put, MethodRouter};
use structopt::StructOpt;
use crate::endpoint::{Service, Source};

mod endpoint;
mod config;
mod cmd;

#[tokio::main]
async fn main() {
    blocking().await;

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

    axum::serve(listener, register_route(services)).await.unwrap();
}

fn register_route(services: Vec<Service>) -> Router {
    let mut app: Router = Router::new();
    let mut routers: Vec<(String, MethodRouter)> = vec![];

    services.into_iter()
        .for_each(|service| routers.append(&mut create_route(service)));

    routers.into_iter()
        .for_each(|route| app = app.clone().route(route.clone().0.as_str(), route.clone().1.clone()));

    app
}

fn create_route(service: Service) -> Vec<(String, MethodRouter)> {
    let sources = service.clone().sources;

    sources.into_iter()
        .map(|source| {
            let endpoint_config = Arc::new(source.clone());
            let route = match source.clone().method.as_str() {
                "GET" => get(move || handle_mock_response(endpoint_config.clone())),
                "POST" => post(move || handle_mock_response(endpoint_config.clone())),
                "PUT" => put(move || handle_mock_response(endpoint_config.clone())),
                "DELETE" => delete(move || handle_mock_response(endpoint_config.clone())),
                "PATCH" => patch(move || handle_mock_response(endpoint_config.clone())),
                _ => unreachable!()
            };

            let endpoint = format!("{}{}", service.clone().prefix, source.clone().endpoint);

            (endpoint, route)
        }).collect()
}

async fn handle_mock_response(
    endpoint_config: Arc<Source>,
) -> impl IntoResponse {
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