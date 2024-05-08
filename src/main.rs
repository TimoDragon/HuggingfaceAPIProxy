mod model;
mod model_handler;
mod chat_completion_request;

use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest, HttpMessage};
use serde::Deserialize;
use crate::chat_completion_request::ChatCompletionRequest;

/*
    This is a small proxy to be abl to use the huggingface api when trying to use the OpenAI API.
    This can be helpful for e.g. Open WebUI if you want to use the huggingface api.
    If you want to add a model just add it in models.json and you are ready to go!
 */
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe { model_handler::load_models().expect("Can't load models"); }

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/v1/models").to(return_models))
            .service(web::resource("/v1/chat/completions").route(web::post().to(handle_chat_request)))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

async fn return_models() -> impl Responder {
    let models = unsafe { model_handler::get_models() };

    println!("Model Get request");

    HttpResponse::Ok().json(models)
}

async fn handle_chat_request(req: HttpRequest, body: web::Bytes) -> impl Responder {
    let body_str = String::from_utf8_lossy(&body);

    let chat_request: ChatCompletionRequest = match serde_json::from_str(&body_str) {
        Ok(request) => request,
        Err(err) => {
            return HttpResponse::BadRequest().body(err.to_string());
        }
    };

    let model = chat_request.get_model().clone();
    let token = req.headers().get("Authorization").unwrap().to_str().unwrap();

    let url = format!("https://api-inference.huggingface.co/models/{}/v1/chat/completions", model);

    let client = reqwest::Client::new();
    let res = client.post(&url)
        .header("Authorization", token)
        .header("Content-Type", "application/json")
        .body(body_str.to_string())
        .send()
        .await;

    println!("{}", url);

    match res {
        Ok(res) => {
            let stream = res.bytes_stream();
            HttpResponse::Ok().streaming(stream)
        }
        Err(err) => {
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}