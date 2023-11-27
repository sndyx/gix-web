use actix_web::{get, HttpResponse, web};

struct AppState {

}

#[get("/{repo}")]
pub async fn repo(
    path: web::Path<String>,
    data: web::Data<AppState>
) -> HttpResponse {
    return HttpResponse::Ok().body("Hello world!");
}