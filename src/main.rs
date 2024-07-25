use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use reqwest::Client;
use std::process::{Command, Child};
use std::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing_actix_web::TracingLogger;
use std::env;

struct AppState {
    client: Client,
    child: Mutex<Option<Child>>,
}

async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn readyz(data: web::Data<AppState>) -> impl Responder {
    let response = data.client.get("http://127.0.0.1:8000/readyz").send().await;
    match response {
        Ok(_) => HttpResponse::Ok().body("READY"),
        Err(_) => HttpResponse::ServiceUnavailable().body("NOT READY"),
    }
}

async fn proxy(req: actix_web::HttpRequest, body: web::Bytes, data: web::Data<AppState>) -> impl Responder {
    let client = &data.client;
    let auth_token = "Bearer ".to_string() + env::var("API_KEY").unwrap().as_str();

    let mut request_builder = client
        .request(req.method().clone(), &format!("http://127.0.0.1:8000{}", req.uri()))
        .headers(req.headers().clone().into())
        .body(body);

    if !auth_token.is_empty() {
        request_builder = request_builder.bearer_auth(auth_token.clone());
    }

    let api_key = req.headers().get("Authorization");
    if api_key.is_none() || api_key.unwrap().to_str().unwrap() != auth_token {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let response = request_builder.send().await;

    match response {
        Ok(mut res) => {
            let mut client_resp = HttpResponse::build(res.status());
            for (key, value) in res.headers().iter() {
                client_resp.insert_header((key.clone(), value.clone()));
            }
            client_resp.body(res.bytes().await.unwrap())
        }
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

async fn start_server() -> std::io::Result<Child> {


    let mut command = Command::new("text-embeddings-router");
    if env::var("MODEL_ID").is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "MODEL_ID is not set"));
    }

    if env::var("API_KEY").is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "API_KEY is not set"));
    }

    let args: Vec<String> = env::args().collect();
    for (key, value) in env::vars() {
        if key == "API_KEY" {
            continue;
        }
        command.env(key, value);
    }

    for arg in &args[1..] {
        command.arg(arg);
    }

    let child = command.spawn()?;
    for _ in 0..90 {
        if std::net::TcpStream::connect("127.0.0.1:8000").is_ok() {
            println!("Service is ready!");
            return Ok(child);
        }
        sleep(Duration::from_secs(1)).await;
    }

    Err(std::io::Error::new(std::io::ErrorKind::Other, "Service failed to start"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::new();
    let child = start_server().await.unwrap();
    let app_state = web::Data::new(AppState {
        client,
        child: Mutex::new(Some(child)),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/healthz", web::get().to(healthz))
            .route("/readyz", web::get().to(readyz))
            .wrap(TracingLogger::default())
            .default_service(web::to(proxy))
    })
    .bind("0.0.0.0:8001")?
    .run()
    .await
}