mod page;
mod middleware;

use std::path::PathBuf;

use actix_web::{get, App, HttpResponse, HttpServer, web};
use rust_embed::RustEmbed;
use crate::middleware::UnwrapRepo;
use crate::page::{index, repo_index, repo_path};

#[derive(RustEmbed)]
#[folder = "templates/css/"]
struct CssFiles;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let path = PathBuf::from("C:\\Users\\Sandy\\IdeaProjects");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(path.clone()))
            .service(css)
            .service(
                match gix::open(path.clone()) {
                    Ok(repo) => web::scope("/")
                        .app_data(web::Data::new(repo)),
                    Err(_) => web::scope("/{repo}")
                }
                .wrap(UnwrapRepo)
                .service(repo_index)
                .service(repo_path)
            )
            .service(index) // Route only active during multi repo mode
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/+css/{file}")]
async fn css(file: web::Path<String>) -> HttpResponse {
    match CssFiles::get(file.into_inner().as_str()) {
        Some(content) => HttpResponse::Ok()
            .content_type("text/css")
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}