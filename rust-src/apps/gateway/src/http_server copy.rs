
use std::{collections::BTreeMap, process::Output, borrow::BorrowMut, future::Future, };

use near_base::{NearResult, NearError};
use tide::{Server,
           security::{CorsMiddleware, Origin},
           http::headers::HeaderValue, Request, Response, StatusCode,
    };

#[derive(Debug)]
pub struct UrlRequest<'a> {
    pub uid: &'a str,
    pub operator: &'a str,
    pub params: BTreeMap<&'a str, &'a str>,
    pub headers: BTreeMap<&'a str, &'a str>,
    pub body: Vec<u8>,
}

#[async_trait::async_trait]
pub trait HttpEventTrait: Sync + Send {
    fn clone_as_event(&self) -> Box<dyn HttpEventTrait>;

    async fn post(&self, req: UrlRequest<'_>) -> NearResult<()>;
    async fn get(&self, req: UrlRequest<'_>) -> NearResult<()>;
}

pub struct HttpServer {
    app: Server<()>,
}

impl HttpServer {
    pub fn new(callback: Box<dyn HttpEventTrait>) -> NearResult<Self> {
        let mut app = tide::new();

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
        app.with(cors);

        // {
        //     let callback = callback.clone_as_event();
        //     app.at("/:uid/:operator").post(move | mut req: Request<()> | {
        //         let callback = callback.clone_as_event();

        //         async move {
        //             let body = req.body_bytes().await.unwrap();

        //             let uid = req.param("uid")?;
        //             let operator = req.param("operator")?;

        //             let mut params = BTreeMap::new();
        //             req.url().query_pairs().for_each(| (k, v)| {
        //                 params.insert(k.as_ref(), v.as_ref());
        //             });

        //             let mut headers = BTreeMap::new();
        //             for name in req.header_names() {
        //                 headers.insert(name.as_str(), req.header(name).map(| val | val.as_str()).unwrap_or(""));
        //             }

        //             let _r = callback.post(UrlRequest { uid, operator, params, headers, body }).await;

        //             return Ok(Response::new(StatusCode::Ok));
        //         }
        //     });
        // }

        {
            let callback = callback.clone_as_event();
            app.at("/:uid/:operator").get(move | mut req: Request<()> | {
                let callback = callback.clone_as_event();
                // let mut req = req;
                async move {
                    let body = req.body_bytes().await.unwrap();

                    let uid = req.param("uid")?;
                    let operator = req.param("operator")?;

                    let mut params = BTreeMap::new();
                    // req.url().query_pairs().for_each(| (k, v)| {
                    //     params.insert(k.as_ref(), v.as_ref());
                    // });

                    let mut headers = BTreeMap::new();
                    for name in req.header_names() {
                        headers.insert(name.as_str(), req.header(name).map(| val | val.as_str()).unwrap_or(""));
                    }

                    let _r = callback.get(UrlRequest { uid, operator, params, headers, body }).await;

                    return Ok(Response::new(StatusCode::Ok));
                }
            });
        }

        Ok(HttpServer{
            app,
        })
    }

    pub async fn start(self, port: u16) -> NearResult<()> {

        self.app
            .listen(format!("0.0.0.0:{}", port))
            .await
            .map_err(| err | {
                NearError::from(err)
            })

    }
}

impl HttpServer {
    async fn call(&self, mut req: Request<()>) -> NearResult<()> {
        let body = req.body_bytes().await.unwrap();

        let uid = req.param("uid").unwrap();
        let operator = req.param("operator").unwrap();

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

        let mut headers = BTreeMap::new();
        for name in req.header_names() {
            headers.insert(name.as_str(), req.header(name).map(| val | val.as_str()).unwrap_or(""));
        }

        let _r = UrlRequest { uid, operator, params, headers, body, };

        Ok(())
    }
    
}
