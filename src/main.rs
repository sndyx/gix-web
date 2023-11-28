mod page;
mod repo;
mod auth;
mod middleware;

use std::future::Future;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use actix_web::{get, App, HttpResponse, HttpServer, web, Resource, HttpMessage, FromRequest, HttpRequest};
use actix_web::body::MessageBody;
use actix_web::dev::{HttpServiceFactory, Payload, Service as _, Service, ServiceFactory, ServiceRequest, ServiceResponse};
use futures_util::future::{err, FutureExt, ok};
use gix::Repository;
use rust_embed::RustEmbed;
use crate::middleware::UnwrapRepo;

struct RepoDir<'a> {
    path: &'a Path
}

struct RepoData<'a> {
    repo: Repository,
    name: &'a str
}

#[derive(RustEmbed)]
#[folder = "templates/css/"]
struct CssFiles;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let path = Path::new("/Users/25alexandercapitos/sndy/Documents");

    HttpServer::new(move || {
        let scope = if let Ok(repo) = gix::open(path) {
            web::scope("")
                .wrap(UnwrapRepo)
                .app_data(RepoData {
                    repo,
                    name: path.file_name().unwrap().to_str().unwrap()
                })
        } else {
            web::scope("{repo}")
                .wrap(UnwrapRepo)
        };

        App::new()
            .app_data(RepoDir { path })
            .service(
                scope
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/css/{file}")]
async fn css(file: web::Path<String>) -> HttpResponse {
    match CssFiles::get(file.into_inner().as_str()) {
        Some(content) => HttpResponse::Ok()
            .content_type("text/css")
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}