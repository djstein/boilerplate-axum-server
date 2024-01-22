use axum::{
    routing::{get, post, MethodRouter},
    Router,
};

fn route(path: &str, method_router: MethodRouter<()>) -> Router {
    Router::new().route(path, method_router)
}

pub fn root() -> Router {
    async fn handler() -> &'static str {
        "Hello, World!"
    }
    route("/", get(handler))
}

pub fn get_foo() -> Router {
    async fn handler() -> &'static str {
        "Hi from `GET /foo`"
    }

    route("/foo", get(handler))
}

pub fn post_foo() -> Router {
    async fn handler() -> &'static str {
        "Hi from `POST /foo`"
    }

    route("/foo", post(handler))
}
