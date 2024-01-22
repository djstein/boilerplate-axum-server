use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode},
    response::IntoResponse,
    Router,
};
use dotenv::dotenv;
use std::time::Duration;
use tower_http::{classify::ServerErrorsFailureClass, cors::CorsLayer, trace::TraceLayer};
use tracing::Span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
use app::routes::{get_foo, post_foo, root};

#[tokio::main]
async fn main() {
    dotenv().ok(); // This line loads the environment variables from the ".env" file.
    let service_addr: String = std::env::var("SERVICE_ADDR").expect("SERVICE_ADDR must be set.");
    let backend_url: String = std::env::var("BACKEND_URL").expect("BACKEND_URL must be set.");
    let frontend_url: String = std::env::var("FRONTEND_URL").expect("FRONTEND_URL must be set.");

    // setup tracing. give it each crate name with a tracing level
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "example=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let origins: [HeaderValue; 2] = [backend_url.parse().unwrap(), frontend_url.parse().unwrap()];

    let backend = async {
        // build our application with a route
        let app = Router::new()
            .merge(root())
            .merge(get_foo())
            .merge(post_foo())
            .layer(
                CorsLayer::new()
                    .allow_credentials(true)
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_origin(origins),
            )
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|_request: &Request<Body>| tracing::debug_span!("http-request"))
                    .on_request(|request: &Request<Body>, _span: &Span| {
                        tracing::debug!("started {} {}", request.method(), request.uri().path());
                    })
                    .on_response(
                        |_response: &Response<Body>, latency: Duration, _span: &Span| {
                            tracing::debug!("response generated in {:?}", latency)
                        },
                    )
                    .on_body_chunk(|chunk: &Bytes, _latency: Duration, _span: &Span| {
                        tracing::debug!("sending {} bytes", chunk.len())
                    })
                    .on_eos(
                        |_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                            tracing::debug!("stream closed after {:?}", stream_duration)
                        },
                    )
                    .on_failure(
                        |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                            tracing::debug!("something went wrong")
                        },
                    ),
            );

        let app = app.fallback(handler_404);

        let listener = tokio::net::TcpListener::bind(service_addr).await.unwrap();
        tracing::debug!(
            "listening on http://localhost:{}",
            listener.local_addr().unwrap().port()
        );
        axum::serve(listener, app).await.unwrap();
    };

    tokio::join!(backend);
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404")
}
