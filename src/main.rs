// Simple server to interact with other servers a bit
// Copyright (C) 2020 bram@neijt.nl

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use actix::fut::wrap_future;
use actix::prelude::*;
use actix::utils::IntervalFunc;
// use actix_files::NamedFile;
use actix_web::client::Client;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::time::Duration;
use std::io;
use std::io::Write;

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
    HttpResponse::Ok().body(include_str!("index.html"))
    // NamedFile::open("src/index.html").unwrap()
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
        let current_action_idx = self.action_idx.fetch_add(1, Ordering::SeqCst);
        let state_mutex = self.state_data.clone();
        let state = self.state_data.read().unwrap();
        let tingle_vec: Vec<Tingle> = state.iter().map(|(_, tingle)| tingle.clone()).collect();
        if tingle_vec.len() > 0 {
            print!(".");
            io::stdout().flush().expect("Could not flush stdout");
            let tingle: Tingle = (*tingle_vec
                .get(current_action_idx % tingle_vec.len())
                .unwrap())
            .clone();
            if tingle.action == "touch" {
                let host: String = tingle.host.clone();

                context.spawn(wrap_future(async move {
                    let tingle: Tingle = (*tingle_vec
                        .get(current_action_idx % tingle_vec.len())
                        .unwrap())
                    .clone();
                    let client = Client::default();
                    let response = client.get(format!("http://{}", host)).send().await;
                    match response {
                        Ok(result) => {
                            let mut state = state_mutex.write().unwrap();
                            state.insert(
                                tingle.host.clone(),
                                Tingle {
                                    result: result
                                        .status()
                                        .canonical_reason()
                                        .unwrap_or("an unknown but good feeling")
                                        .to_string(),
                                    ..tingle
                                },
                            );
                        }

                        Err(_) => {
                            let mut state = state_mutex.write().unwrap();
                            state.insert(
                                tingle.host.clone(),
                                Tingle {
                                    result: String::from("a feeling of rejection"),
                                    ..tingle
                                },
                            );
                        }
                    };
                }));
            }
        };
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
    println!("Starting at port {}", port);
    let state: HashMap<String, Tingle> = HashMap::new();
    let state_data = web::Data::new(RwLock::new(state));
    let state_data_clone = state_data.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            .route("/", web::get().to(index))
            .route("/state", web::get().to(get_state))
            .route("/touch/{party}", web::get().to(touch))
    })
    .bind(format!("{}:{}", address, port))
    .expect("Could not bind to address!")
    .run();
    let action_idx = web::Data::new(AtomicUsize::new(0usize));
    ToucherFeeler::create(|_ctx: &mut Context<ToucherFeeler>| ToucherFeeler {
        state_data: state_data_clone,
        action_idx: action_idx,
    });

    server.await
}
