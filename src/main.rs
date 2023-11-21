mod page;

use actix_web::{get, App, HttpResponse, HttpServer, web, HttpRequest};
use crate::page::index;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
//          .app_data(web::Data::new(RepoData {
//              path: String::from("/Users/25alexandercapitos/sndy/Documents"),
//          }))
            .service(index)
            .service(css)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/css/{file}")]
pub async fn css(req: HttpRequest) -> HttpResponse {
    let file_name: String = req.match_info().get("file").unwrap().parse().unwrap();
    let file = match file_name.as_str() {
        "index.css" => String::from_utf8_lossy(include_bytes!("../css/index.css")),
        "example.css" => String::from_utf8_lossy(include_bytes!("../css/file.css")),
        _ => return HttpResponse::NotFound().body("File not found.")
    };
    HttpResponse::Ok()
        .content_type("text/css")
        .body(file)
}