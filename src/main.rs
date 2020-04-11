use actix::System;
use actix_files::NamedFile;
use actix_web::client::Client;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::time::Duration;
use actix::prelude::*;
use actix::utils::IntervalFunc;

#[derive(Clone, Serialize)]
struct Tingle {
    action: String,
    host: String,
    result: String,
}

async fn index(
    req: HttpRequest,
    state_data: web::Data<RwLock<HashMap<String, Tingle>>>,
) -> impl Responder {
    let host = format!("{}", req.peer_addr().unwrap().ip());
    let mut state = state_data.write().unwrap();
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

async fn get_state(state_data: web::Data<RwLock<HashMap<String, Tingle>>>) -> impl Responder {
    let state = state_data.read().unwrap();
    let response: Vec<Tingle> = state.iter().map(|(_, tingle)| tingle.clone()).collect();
    HttpResponse::Ok().json(response)
}

async fn touch(
    party: web::Path<String>,
    state_data: web::Data<RwLock<HashMap<String, Tingle>>>,
) -> impl Responder {
    let mut state = state_data.write().unwrap();
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

struct ToucherFeeler {
    action_idx: web::Data<AtomicUsize>,
    state_data: web::Data<RwLock<HashMap<String, Tingle>>>,
}

impl ToucherFeeler {
    fn tick(&mut self, context: &mut Context<Self>) {
        println!("tick");
        let current_action_idx = self.action_idx.fetch_add(1, Ordering::SeqCst);
        println!("Acting on {}", current_action_idx);
        let mut state = self.state_data.write().unwrap();
        let tingle_vec: Vec<Tingle> = state.iter().map(|(_, tingle)| tingle.clone()).collect();
        if tingle_vec.len() > 0 {
            let client = Client::default();
            let tingle: Tingle = (*tingle_vec
                .get(current_action_idx % tingle_vec.len())
                .unwrap())
            .clone();
            if tingle.action == "touch" {
                let host: String = tingle.host.clone();
                // let reponse = client.get(format!("http://{}", host)).send().unwrap();
                // let mut client = Client::new();
                // client
                //     .get("https://www.rust-lang.org") // <- Create request builder
                //     .header("User-Agent", "Actix-web")
                //     .send() // <- Send http request
                //     // .map_err(Error::from)
                //     .map(|res| {
                //         HttpResponse::Ok()
                //             .content_type("application/json")
                //             .body("{}")
                //     })
                // match response {
                //     Ok(result) => state.insert(
                //         tingle.host.clone(),
                //         Tingle {
                //             result: result
                //                 .status()
                //                 .canonical_reason()
                //                 .unwrap_or("an unknown but good feeling")
                //                 .to_string(),
                //             ..tingle
                //         },
                //     ),
                //     Err(_) => state.insert(
                //         tingle.host.clone(),
                //         Tingle {
                //             result: String::from("a feeling of rejection"),
                //             ..tingle
                //         },
                //     ),
                // println!("Touch");
                // }
            }
        };
        println!("tick")
    }
}

impl Actor for ToucherFeeler {
   type Context = Context<Self>;

   fn started(&mut self, context: &mut Context<Self>) {
       // spawn an interval stream into our context
       IntervalFunc::new(Duration::from_secs(3), Self::tick)
           .finish()
           .spawn(context);
   }
}



#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let address = "0.0.0.0";
    let port = env::var("PORT").unwrap_or(String::from("8080"));

    let state: HashMap<String, Tingle> = HashMap::new();
    let state_data = web::Data::new(RwLock::new(state));
    let state_data_clone = state_data.clone();
    let action_idx = web::Data::new(AtomicUsize::new(0usize));
    let action_idx_clone = action_idx.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            .app_data(action_idx.clone())
            .route("/", web::get().to(index))
            .route("/state", web::get().to(get_state))
            .route("/touch/{party}", web::get().to(touch))
    })
    .bind(format!("{}:{}", address, port))
    .expect("Could not bind to address!")
    .run();
    
    ToucherFeeler::create(|ctx: &mut Context<ToucherFeeler>| {
        ToucherFeeler{
            state_data: state_data_clone,
            action_idx: action_idx_clone,
        }
    });

    server.await

}
