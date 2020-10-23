use actix_web::{
    body::Body,
    get,
    web,
    App,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder
};
use listenfd::ListenFd;
use mime_guess::from_path;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::env;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

/// A generic handler that serves from the embed static files
fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let body: Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };
            HttpResponse::Ok().content_type(from_path(path).first_or_octet_stream().as_ref()).body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

/// An actix responder to serve the index
fn index() -> HttpResponse {
    handle_embedded_file("index.html")
}

/// An actix responder to serve static files
#[get("/{_:.*}")]
fn static_files(path: web::Path<String>) -> HttpResponse {
    handle_embedded_file(&path.0)
}

/// An actix responder that serves the wasm.js file generated for the frontend
#[get("/wasm.js")]
async fn yew_app_js(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(include_str!(env!("MY_LITTLE_ROBOTS_JS")))
}

/// An actix responder that serves the wasm.wasm file generated for the frontend
#[get("/wasm_bg.wasm")]
async fn yew_app_wasm(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/wasm")
        .body(web::Bytes::from_static(include_bytes!(env!(
            "MY_LITTLE_ROBOTS_WASM"
        ))))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    // Initialize ListenFd to enable auto reloading of the server
    let mut listenfd = ListenFd::from_env();

    // Construct the server
    let mut server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .service(yew_app_js)
            .service(yew_app_wasm)
            .service(static_files)
    });

    // Tell the server how to listen for trafic. If `systemfd` is used to start the server we use
    // that socket, otherwise we simply bind to the default bind address.
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3030")?
    };

    // Run the server
    server.run().await
}
