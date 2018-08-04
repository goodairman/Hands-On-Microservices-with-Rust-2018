extern crate futures;
extern crate hyper;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::ops::Range;
use futures::{future, Future, Stream};
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;
use rand::Rng;
use rand::distributions::{Bernoulli, Normal, Uniform};

static INDEX: &[u8] = b"Random Microservice";

#[derive(Deserialize)]
#[serde(tag = "distribution", content = "parameters", rename_all = "lowercase")]
enum RngRequest {
    Uniform {
        #[serde(flatten)]
        range: Range<i32>,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Bernoulli {
        p: f64,
    },
}

#[derive(Serialize)]
struct RngResponse {
    value: f64,
}

fn microservice_handler(req: Request<Body>)
    -> Box<Future<Item=Response<Body>, Error=Error> + Send>
{
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/random") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/random") => {
            let body = req.into_body().concat2()
                .and_then(|chunks| {
                    let request = serde_json::from_slice::<RngRequest>(chunks.as_ref());
                    let mut rng = rand::thread_rng();
                    let value = {
                        match request {
                            Ok(RngRequest::Uniform { range }) => {
                                rng.sample(Uniform::from(range)) as f64
                            },
                            Ok(RngRequest::Normal { mean, std_dev }) => {
                                rng.sample(Normal::new(mean, std_dev)) as f64
                            },
                            Ok(RngRequest::Bernoulli { p }) => {
                                rng.sample(Bernoulli::new(p)) as i8 as f64
                            },
                            Err(err) => {
                                let resp = Response::builder()
                                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                                    .body(err.to_string().into())
                                    .unwrap();
                                return Ok(resp);
                            },
                        }
                    };
                    let resp = RngResponse { value };
                    let body = serde_json::to_string(&resp);
                    match body {
                        Ok(body) => {
                            Ok(Response::new(body.into()))
                        },
                        Err(err) => {
                            let resp = Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(err.to_string().into())
                                .unwrap();
                            Ok(resp)
                        },
                    }
                });
            Box::new(body)
        },
        _ => {
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("".into())
                .unwrap();
            Box::new(future::ok(resp))
        },
    }
}

fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| {
        service_fn(microservice_handler)
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}