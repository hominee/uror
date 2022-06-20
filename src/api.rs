use crate::entity::*;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, StatusCode,
};
use std::collections::HashMap;
use std::env;

type GenericError = Box<dyn std::error::Error + Send + Sync>;

pub fn get_params<T: hyper::body::Buf>(
    uri_: &hyper::Uri,
    headers: &hyper::HeaderMap,
    bytes: &T,
) -> HashMap<String, String> {
    let mut params = url::form_urlencoded::parse(uri_.query().unwrap_or("").as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let content_type = params.get("uri");
    if content_type.is_none() {
        match headers
            .get("content-type")
            .and_then(|en| Some(en.to_str().unwrap()))
        {
            Some("application/json") => {
                let uri: Uri = serde_json::from_slice(bytes.chunk()).unwrap();
                log::debug!("json params uri: {}", &uri.uri);
                params.insert("uri".into(), uri.uri);
            }

            Some("application/x-www-form-urlencoded") => {
                url::form_urlencoded::parse(bytes.chunk())
                    .into_owned()
                    .for_each(|(k, v)| {
                        log::debug!("params urlencoded key: {}, value: {}", k, v);
                        params.insert(k, v);
                    });
            }
            _ => {}
        }
    } else {
        log::debug!("params uri: {:?}", params.get("uri").as_ref().unwrap());
    }
    params
}

async fn serve_actor(req: Request<Body>, mut actor: Actor) -> Result<Response<Body>, GenericError> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::POST, "/") => {
            let (headers, body) = req.into_parts();
            let bytes = hyper::body::aggregate(body).await?;
            let params = get_params(&headers.uri, &headers.headers, &bytes);
            let uri = params.get("uri");
            log::debug!("uri to insert: {}", uri.as_ref().unwrap());
            if uri.is_none() {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(r#"{"ok":false,"desc":"uri is empty"}"#.into())
                    .unwrap());
            }
            actor
                .insert(uri.unwrap().to_owned())
                .and_then(|en| {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from(en))
                        .unwrap())
                })
                .map_err(|e| e.into())
        }
        (&Method::GET, _) => {
            let path = req.uri().path().strip_prefix("/").unwrap();
            log::debug!("uri to read: {}", &path);
            if path.contains("/") {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(r#"{"ok":false,"desc":"invalid uri"}"#.into())
                    .unwrap());
            }
            actor
                .read(path)
                .and_then(|en| {
                    Ok(Response::builder()
                        .status(302)
                        .header("Location", en)
                        .body(Body::empty())
                        .unwrap())
                })
                .map_err(|e| e.into())
        }
        (&Method::DELETE, "/") => {
            let (headers, body) = req.into_parts();
            let bytes = hyper::body::to_bytes(body).await?;
            let params = get_params(&headers.uri, &headers.headers, &bytes);
            let uri = params.get("uri");
            if uri.is_none() {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(r#"{"ok":false,"desc":"uri is empty"}"#.into())
                    .unwrap());
            }
            log::debug!("uri to delete: {}", uri.as_ref().unwrap());
            actor
                .delete(uri.unwrap().to_owned())
                .and_then(|_| {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(r#"{"ok":true}"#.into())
                        .unwrap())
                })
                .map_err(|e| e.into())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Invald Request".into())
            .unwrap()),
    }
}

#[tokio::main]
pub async fn main() -> Result<(), GenericError> {
    dotenv::dotenv().ok();
    #[cfg(feature = "logger")]
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let actor = Actor::new();
    let service = make_service_fn(move |_| {
        let actor = actor.clone();
        async { Ok::<_, GenericError>(service_fn(move |req| serve_actor(req, actor.clone()))) }
    });

    let addr = env::var("ADDR").expect("ADDR not set").parse().unwrap();
    let server = hyper::Server::bind(&addr).serve(service);
    log::info!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}

#[test]
fn test_params() {
    let s = r#"{"uri": "https://example.com", "arr": [12, 1]}"#;
    let r: Result<Uri, _> = serde_json::from_str(s);
    assert!(r.is_ok());
    dbg!(&r);
    let s1 = "uri=https://example.com&id=0";
    let r1 = url::form_urlencoded::parse(s1.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();
    assert!(r1.get("uri").is_some());
    dbg!(&r1);
    //assert!(false);
}
