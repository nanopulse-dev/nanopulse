use anyhow::Result;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    response::Response,
    routing::any,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use http::{
    StatusCode, Uri,
    header::{self, HeaderMap, HeaderValue},
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, info, trace};
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use crate::config;
use crate::integration;

mod device;
mod gateway;
mod region;
mod workspace;

#[derive(RustEmbed)]
#[folder = "./ui/dist"]
struct Asset;

#[derive(OpenApi)]
struct ApiDoc;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct GetBynameParams {
    /// Name of the object to get.
    pub name: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListRequest {
    /// The offset in the result set.
    #[param(default = 0)]
    pub offset: Option<usize>,

    /// The number of items to return.
    #[param(default = 10)]
    pub limit: Option<usize>,
}

#[derive(Serialize, ToSchema)]
struct ListResponse<T> {
    pub total_count: usize,
    pub result: Vec<T>,
}

// 1. Define an error schema for your API
#[derive(Serialize, ToSchema)]
pub struct ApiError {
    /// Error message describing what went wrong
    pub message: String,
}

pub async fn setup() -> Result<()> {
    let conf = config::get();

    info!(bind = %conf.api.bind, "Starting API and web-interface");

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(workspace::workspace_create))
        .routes(routes!(workspace::workspace_get))
        .routes(routes!(workspace::workspace_update))
        .routes(routes!(workspace::workspace_delete))
        .routes(routes!(workspace::workspace_list))
        .routes(routes!(region::region_list))
        .routes(routes!(gateway::gateway_create))
        .routes(routes!(gateway::gateway_get))
        .routes(routes!(gateway::gateway_update))
        .routes(routes!(gateway::gateway_delete))
        .routes(routes!(gateway::gateway_list))
        .routes(routes!(gateway::gateway_allow_key_exchange))
        .routes(routes!(device::device_create))
        .routes(routes!(device::device_get))
        .routes(routes!(device::device_update))
        .routes(routes!(device::device_delete))
        .routes(routes!(device::device_list))
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/api").url("/apidoc/openapi.json", api));
    let router = router.route("/api/ws", any(ws_handler));
    let router = router.fallback(static_handler);
    let router = router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    );

    let listener = tokio::net::TcpListener::bind(&conf.api.bind).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

struct TaskGuard<T>(Option<JoinHandle<T>>);

impl<T> TaskGuard<T> {
    fn new(j: JoinHandle<T>) -> Self {
        TaskGuard(Some(j))
    }
}

impl<T> Drop for TaskGuard<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            trace!("TaskGuard dropped, aborting handle");
            handle.abort();
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(ws_handle_socket)
}

async fn ws_handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn({
        let tx = tx.clone();

        async move {
            let mut _not_handle = None;

            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(v) => {
                        if let Ok(pl) =
                            serde_json::from_str::<integration::internal::Subscribe>(v.as_str())
                            && let Ok(mut chan) =
                                integration::internal::get_channel(&pl.workspace_name)
                        {
                            _not_handle = Some(TaskGuard::new(tokio::spawn({
                                let tx = tx.clone();

                                async move {
                                    while let Ok(msg) = chan.recv().await {
                                        if let Ok(msg) = serde_json::to_string(&msg) {
                                            trace!("sending notification, payload: {}", msg);
                                            _ = tx.send(Message::Text(msg.into())).await;
                                        }
                                    }
                                }
                            })));
                        }
                    }
                    Message::Ping(v) => _ = tx.send(Message::Pong(v)).await,
                    _ => {}
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    };
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = {
        let mut chars = uri.path().chars();
        chars.next();
        chars.as_str()
    };
    if path.is_empty() {
        path = "index.html";
    }

    if let Some(asset) = Asset::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime.as_ref()).unwrap(),
        );
        (StatusCode::OK, headers, asset.data.into())
    } else {
        (StatusCode::NOT_FOUND, HeaderMap::new(), vec![])
    }
}
