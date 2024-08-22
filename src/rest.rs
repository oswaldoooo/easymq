use std::sync::Arc;

use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Extension, Json,
};

use crate::log;

pub async fn run<T, B>(
    svc: Arc<super::MessageQueueManager<T, B>>,
    bind: String,
    prefix: Option<String>,
) where
    T: log::Log + Send + 'static,
    B: log::LogBuilder<T> + Send + 'static,
{
    let mut router = axum::Router::new();
    if let Some(prefix) = prefix {
        router = router.nest(prefix.as_str(), route_init::<T, B>(axum::Router::new()));
    } else {
        router = route_init::<T, B>(router);
    }
    router=router.layer(axum::Extension(svc));
    // let router=router.layer(axum::Extension(svc.clone()));
    let listener = tokio::net::TcpListener::bind(bind.as_str())
        .await
        .expect("bind tcp failed");
    axum::serve(listener, router)
        .await
        .expect("serve http failed");
}
fn route_init<T, B>(mut router: axum::Router) -> axum::Router
where
    T: log::Log + Send + 'static,
    B: log::LogBuilder<T> + Send + 'static,
{
    router = router
        .route("/publish", post(publish::<T, B>))
        .route("/read", get(readmsg::<T, B>));
    router
}
async fn publish<T, B>(
    Extension(svc): Extension<Arc<super::MessageQueueManager<T, B>>>,
    Json(req): Json<args::Message>,
) -> (StatusCode, Json<resp::Response<bool>>)
where
    T: log::Log + Send + 'static,
    B: log::LogBuilder<T> + Send + 'static,
{
    if let Err(err) = svc.push(req.topic, req.content).await {
        //record log
        eprintln!("push msg failed {err}");
        //
        return (
            StatusCode::OK,
            Json(resp::Response::error("operation failed")),
        );
    }
    (StatusCode::OK, Json(resp::Response::ok(true)))
}
async fn readmsg<T, B>(
    Extension(svc): Extension<Arc<super::MessageQueueManager<T, B>>>,
    Query(req): Query<args::ReadMsg>,
) -> (StatusCode, Json<resp::Response<resp::Message>>)
where
    T: log::Log + Send + 'static,
    B: log::LogBuilder<T> + Send + 'static,
{
    let result = svc.read_latest(req.topic).await;
    if let Err(err) = result {
        //record log
        eprintln!("read msg failed {err}");
        //
        return (
            StatusCode::BAD_REQUEST,
            Json(resp::Response::error("read msg failed")),
        );
    }
    let mut data = result.unwrap();
    if let Err(err) = data.ack().await {
        //record log
        eprintln!("ack failed {err}")
        //
    }
    (
        StatusCode::OK,
        Json(resp::Response::ok(resp::Message {
            id: data.msg_id,
            topic: data.topic,
            content: data.data,
        })),
    )
}
mod args {
    #[derive(serde::Deserialize)]
    pub struct ReadMsg {
        pub topic: String,
    }
    #[derive(serde::Deserialize)]
    pub struct Message {
        pub topic: String,
        pub content: String,
    }
}
mod resp {
    #[derive(serde::Serialize)]
    pub struct Response<T> {
        ok: bool,
        reason: Option<&'static str>,
        data: Option<T>,
    }
    impl<T> Response<T> {
        pub fn ok(data: T) -> Self {
            Self {
                ok: true,
                reason: None,
                data: Some(data),
            }
        }
        pub fn error(reason: &'static str) -> Self {
            Self {
                ok: false,
                reason: Some(reason),
                data: None,
            }
        }
    }

    #[derive(serde::Serialize)]
    pub struct Message {
        pub id: u32,
        pub topic: String,
        pub content: String,
    }
}
