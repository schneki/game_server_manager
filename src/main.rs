extern crate hyper;
extern crate futures;
extern crate zip;

extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;


mod minecraft;

use futures::future;

use hyper::{Body, Method, Response, Request, Server, StatusCode};
use hyper::service::service_fn;
use hyper::header::{HeaderMap, CONTENT_DISPOSITION};
use hyper::rt::{self, Future, Stream};

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;


use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};

use std::process::Child;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub mc_version: String,
    pub mc_server_jar: String,
    pub mc_path: String,
    pub mc_world_name: String 
}

use std::fs;

impl Config {
    pub fn load() -> Config {
        let mut f = fs::File::open("config.json").unwrap();
        serde_json::from_reader(f).unwrap() 
    }
}

pub fn handler(req: Request<Body>, 
               counter: &Arc<AtomicUsize>,
               config: &Config,
               mc_server: &Arc<Mutex<Option<Child>>>) -> BoxFut {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/api/download/mods") => {
            let buffer = minecraft::get_mods_zip(&config.mc_path);
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_DISPOSITION,
                "Content-Disposition: attachment; filename=mods.zip".parse().unwrap()
            );
            *response.headers_mut() = headers;
            *response.body_mut() = Body::from(buffer);

        }
        (&Method::GET, "/api/blank") => {
            counter.fetch_add(1, Ordering::AcqRel);
            *response.body_mut() = Body::from("blank");

        }
        (&Method::GET, "/api/mc_start") => {
            match minecraft::start_server(&config) {
                Ok(child) => {
                    minecraft::zip_mods(&config.mc_path, minecraft::get_mods(&config.mc_path));
                    *mc_server.lock().unwrap() = Some(child);
                    *response.body_mut() = Body::from("server started")
                }
                Err(err) => {
                    *response.status_mut() = StatusCode::PRECONDITION_FAILED;
                    *response.body_mut() = Body::from(err)
                }

            }
        }
        (&Method::GET, "/api/mc_stop") => {
            let mut mc_server = &mut *mc_server.lock().unwrap();
            match mc_server {
                Some(ref mut mc) => {
                    minecraft::stop_server(mc);
                    *response.body_mut() = Body::from("minecraft server stopped");
                }
                None => {
                    *response.status_mut() = StatusCode::PRECONDITION_FAILED;
                    *response.body_mut() = Body::from("minecraft server not running");
                }
            }

        }
        _ => { *response.status_mut() = StatusCode::NOT_FOUND; }

    }
    Box::new(future::ok(response))
}

fn start_server() {
    let addr = ([127, 0, 0, 1], 5000).into();

    let counter = Arc::new(AtomicUsize::new(0));
    let mc_server = Arc::new(Mutex::new(None));
    let config = Config::load();
    
    let server = Server::bind(&addr)
        .serve(move || {
            let counter = counter.clone();
            let mc_server = mc_server.clone();
            let config = config.clone();
            service_fn(move |req| {handler(req, &counter, &config, &mc_server)})
        })
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listenig on http://{}", addr);
    hyper::rt::run(server);
}

fn main() {
    start_server();
}
