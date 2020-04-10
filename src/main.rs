use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

#[derive(Clone, Serialize)]
struct Tingle {
    action: String,
    host: String,
    result: String,
}


async fn index(
    req: HttpRequest,
    state_data: web::Data<Mutex<HashMap<String, Tingle>>>,
) -> impl Responder  {
    let host = format!("{}", req.peer_addr().unwrap().ip());
    let mut state = state_data.lock().unwrap();
    state.insert(
        host.clone(),
        Tingle {
            action: "felt".into(),
            host: host,
            result: "nice warm tingles".into(),
        },
    );
    // HttpResponse::Ok().body(include_str!("index.html"))
    NamedFile::open("src/index.html").unwrap()
}

async fn get_state(state_data: web::Data<Mutex<HashMap<String, Tingle>>>) -> impl Responder {
    let state = state_data.lock().unwrap();
    let response: Vec<Tingle> = state.iter().map(|(_, tingle)| tingle.clone()).collect();
    HttpResponse::Ok().json(response)
}

async fn touch(
    party: web::Path<String>,
    state_data: web::Data<Mutex<HashMap<String, Tingle>>>,
) -> impl Responder {
    let mut state = state_data.lock().unwrap();
    state.insert(
        party.clone(),
        Tingle {
            action: "touched".into(),
            host: party.clone(),
            result: "nothing yet".into(),
        },
    );
    HttpResponse::Ok().json("rocking")
}



#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    let address = "0.0.0.0";
    let port = env::var("PORT").unwrap_or(String::from("8080"));

    let state: HashMap<String, Tingle> = HashMap::new();
    let state_data = web::Data::new(Mutex::new(state));

    HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            .route("/", web::get().to(index))
            .route("/state", web::get().to(get_state))
            .route("/touch/{party}", web::get().to(touch))
    })
    .bind(format!("{}:{}", address, port))?
    .run()
    .await
}
