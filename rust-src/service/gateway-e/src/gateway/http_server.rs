
use std::{collections::BTreeMap, };

use tide::{Server,
           security::{CorsMiddleware, Origin},
           http::headers::HeaderValue, Request, Response, StatusCode, Body,
    };

use near_base::{NearResult, NearError};
use near_core::get_data_path;

use super::{p::ToHttpResult, App};

#[derive(Debug)]
pub struct UrlRequest<'a> {
    pub uid: &'a str,
    pub operator: &'a str,
    pub params: BTreeMap<&'a str, &'a str>,
    pub header: BTreeMap<&'a str, &'a str>,
    pub body: Body,
}

impl<'a> UrlRequest<'a> {
    async fn to_request(req: &'a mut Request<()>) -> NearResult<UrlRequest<'_>> {
        let body = req.take_body();

        let uid = req.param("uid").unwrap();
        let operator = req.param("operator").unwrap();

        // params from url
        let mut params = BTreeMap::new();
        {
            let query_vec: Vec<&str> = if let Some(query_str) = req.url().query() {
                query_str.split('&').collect()
            } else {
                vec![]
            };

            for item in query_vec {
                item.split_once('=')
                    .map(| (k, v) |{
                        params.entry(k).or_insert(v);
                    });
            }
        }

        let mut header = BTreeMap::new();
        for name in req.header_names() {
            header.insert(name.as_str(), req.header(name).map(| val | val.as_str()).unwrap_or(""));
        }

        Ok(UrlRequest { uid, operator, params, header, body, })
    }

}

#[async_trait::async_trait]
pub trait HttpEventTrait: Sync + Send {
    async fn post(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>>;
    async fn get(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>>;
}

pub struct HttpServer {
    #[allow(unused)]
    app: App,
    server: Server<()>,
}

impl HttpServer {
    pub fn new(app: App) -> NearResult<Self> {
        let mut server = tide::new();

        let cors =
            CorsMiddleware::new()
                .allow_methods(
                    "GET, POST, PUT, DELETE, OPTIONS"
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .allow_origin(Origin::from("*"))
                .allow_credentials(true)
                .allow_headers("*".parse::<HeaderValue>().unwrap())
                .expose_headers("*".parse::<HeaderValue>().unwrap());
        server.with(cors);

        {
            let r = move | mut req: Request<()> | {
                let app = app.clone();

                async move {
                    let r = {
                        let r = UrlRequest::to_request(&mut req).await.unwrap();

                        match r.operator {
                            "index" | "index.html" => {
                                let mut r = Response::new(StatusCode::Ok);
                                r.set_body(tide::Body::from_file(get_data_path().join("index.html").as_path()).await.unwrap());
                                r
                            }
                            _ => {
                                let r = 
                                    app.post(r)
                                        .await
                                        .map_err(| err | {
                                            // error!("")
                                            println!("failed get http request with err {}", err);
                                            err
                                        });

                                if let Ok(r) = r {
                                    r.to_result().into_response().await
                                } else {
                                    Response::new(StatusCode::BadRequest)
                                }
                            }
                        }
                    };

                    return Ok(r);
                }
            };
        }
        {
            let app = app.clone();
            server.at("/:uid/:operator").post(move | mut req: Request<()> | {
                let app = app.clone();

                async move {
                    let r = {
                        let r = UrlRequest::to_request(&mut req).await.unwrap();

                        match r.operator {
                            "index" | "index.html" => {
                                let mut r = Response::new(StatusCode::Ok);
                                r.set_body(tide::Body::from_file(get_data_path().join("index.html").as_path()).await.unwrap());
                                r
                            }
                            _ => {
                                let r = 
                                    app.post(r)
                                        .await
                                        .map_err(| err | {
                                            // error!("")
                                            println!("failed get http request with err {}", err);
                                            err
                                        });

                                if let Ok(r) = r {
                                    r.to_result().into_response().await
                                } else {
                                    Response::new(StatusCode::BadRequest)
                                }
                            }
                        }
                    };

                    return Ok(r);
                }
            });
        }

        {
            let app = app.clone();
            server.at("/:uid/:operator").get(move | mut req: Request<()> | {
                let app = app.clone();

                async move {
                    let r = {
                        let r = UrlRequest::to_request(&mut req).await.unwrap();

                        match r.operator {
                            "index" | "index.html" => {
                                let mut r = Response::new(StatusCode::Ok);
                                r.set_body(tide::Body::from_file(get_data_path().join("index.html").as_path()).await.unwrap());
                                r
                            }
                            _ => {
                                let r = 
                                    app.get(r)
                                        .await
                                        .map_err(| err | {
                                            // error!("")
                                            println!("failed get http request with err {}", err);
                                            err
                                        });

                                if let Ok(r) = r {
                                    r.to_result().into_response().await
                                } else {
                                    Response::new(StatusCode::BadRequest)
                                }
                            }
                        }
                    };

                    return Ok(r);
                }
            });
        }

        // {
        //     // let callback = callback.clone_as_event();
        //     server.at("/:uid/index.html").get(move | _: Request<()> | {
        //         // let callback = callback.clone_as_event();

        //         async move {
        //             let mut resp = Response::new(StatusCode::Ok);
        //             resp.set_body(tide::Body::from_file(get_data_path().join("index.html").as_path()).await.unwrap());
        //             return Ok(resp);
        //         }
        //     });

        // }

        Ok(HttpServer{
            app: app.clone(),
            server,
        })
    }

    pub async fn start(self, port: u16) -> NearResult<()> {

        self.server
            .listen(format!("0.0.0.0:{}", port))
            .await
            .map_err(| err | {
                NearError::from(err)
            })

    }
}
