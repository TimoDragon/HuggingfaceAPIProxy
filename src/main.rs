mod model;
mod model_handler;
mod chat_completion_request;

use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest, HttpMessage};
use actix_web::web::{Bytes};
use futures::{Stream, StreamExt, TryStreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use serde_json::value::Index;
use crate::chat_completion_request::ChatCompletionRequest;

/*
    This is a small proxy to be abl to use the huggingface api when trying to use the OpenAI API.
    This can be helpful for e.g. Open WebUI if you want to use the huggingface api.
    If you want to add a model just add it in models.json, and you are ready to go!
 */
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe { model_handler::load_models().expect("Can't load models"); }

    println!("Listening for connections");

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

    println!("{}", url);
    let _ = req.headers().iter().map(|h| {
        println!("{:?}: {:?}", h.0, h.1);
    });

    let mut stream: bool = false;
    let mut body_json: Value = serde_json::from_str(&body_str).unwrap();

    println!("{}", body_str);

    /*if let Some(obj) = body_json.as_object_mut() {
        obj.remove("logit_bias");

        if let Some(s) = obj.get_mut("stream") {
            stream = s.as_bool().unwrap();
        }
    }*/

    let body_str = serde_json::to_string(&body_json).unwrap();

    let client = reqwest::Client::new();
    let res = client.post(&url)
        .header("Authorization", token)
        .header("Content-Type", "application/json")
        .header("x-use-cache", 0)
        .body(body_str)
        .send()
        .await;

    if stream {
        match res {
            Ok(res) => {
                let stream = res.bytes_stream();

                let mut response = HttpResponse::Ok();
                response.append_header(("Content-Type", "text/event-stream"));

                let uuid = Uuid::new_v4();

                // iterate through the stream
                let stream_reader = stream.map_ok(move |chunk| {
                    let json_str = String::from_utf8_lossy(&chunk);

                    // trim the string so that it can be used
                    if json_str.trim() == ":" {
                        return Bytes::from("");
                    }

                    let json_str = json_str.trim_start_matches("data:").trim_end_matches("\n\n");

                    // convert the json string to json format
                    let mut json: Value = match serde_json::from_str(json_str) {
                        Ok(json) => json,
                        Err(err) => {
                            println!("Error: {}", err);
                            return Bytes::from(err.to_string().as_bytes().to_vec());
                        }
                    };
                    
                    println!("{}", json_str);

                    // check if the json contains an error
                    if let Some(obj) = json.as_object_mut() {
                        if let Some(err) = obj.get_mut("error") {
                            println!("Error: {}", json_str);
                            return Bytes::from(json_str.as_bytes().to_vec());
                        }
                    }

                    // make a string out of the json and stream it to the user
                    let mut json_str = serde_json::to_string(&json).unwrap();
                    json_str.insert_str(0, "data:");
                    json_str.push_str("\n\n");

                    Bytes::from(json_str.as_bytes().to_vec())
                });

                response.streaming(stream_reader)
            }
            Err(err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
        }
    } else {
        match res {
            Ok(res) => {
                HttpResponse::Ok().body(res.text().await.unwrap())
            }
            Err(err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
        }
    }
}