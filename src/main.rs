use actix_files::NamedFile;
use actix_web::client::Client;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
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
) -> impl Responder {
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
            action: "touch".into(),
            host: party.clone(),
            result: "nothing yet".into(),
        },
    );
    HttpResponse::Ok().json("rocking")
}

async fn run(
    action_idx: web::Data<AtomicUsize>,
    state_data: web::Data<Mutex<HashMap<String, Tingle>>>,
) -> impl Responder {
    let current_action_idx = action_idx.fetch_add(1, Ordering::SeqCst);
    println!("Acting on {}", current_action_idx);
    let mut state = state_data.lock().unwrap();
    let tingle_vec: Vec<Tingle> = state.iter().map(|(_, tingle)| tingle.clone()).collect();
    if tingle_vec.len() > 0 {
        let client = Client::default();
        let tingle: Tingle = (*tingle_vec
            .get(current_action_idx % tingle_vec.len())
            .unwrap())
        .clone();
        if tingle.action == "touch" {
            let host: String = tingle.host.clone();
            let response = client.get(format!("http://{}", host)).send().await;

            match response {
                Ok(result) => state.insert(
                    tingle.host.clone(),
                    Tingle {
                        result: result
                            .status()
                            .canonical_reason()
                            .unwrap_or("an unknown but good feeling")
                            .to_string(),
                        ..tingle
                    },
                ),
                Err(_) => state.insert(
                    tingle.host.clone(),
                    Tingle {
                        result: String::from("a feeling of rejection"),
                        ..tingle
                    },
                ),
            };
            HttpResponse::Ok().json("ok")
        } else {
            HttpResponse::Ok().json("fail")
        }
    } else {
        HttpResponse::Ok().json("nop")
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let address = "0.0.0.0";
    let port = env::var("PORT").unwrap_or(String::from("8080"));

    let state: HashMap<String, Tingle> = HashMap::new();
    let state_data = web::Data::new(Mutex::new(state));
    let action_idx = web::Data::new(AtomicUsize::new(0usize));

    HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            .app_data(action_idx.clone())
            .route("/", web::get().to(index))
            .route("/run", web::get().to(run))
            .route("/state", web::get().to(get_state))
            .route("/touch/{party}", web::get().to(touch))
    })
    .bind(format!("{}:{}", address, port))?
    .run()
    .await
}
