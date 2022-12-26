use actix_web::web::Form;
use actix_web::HttpResponse;

pub async fn subscribe(form: Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
