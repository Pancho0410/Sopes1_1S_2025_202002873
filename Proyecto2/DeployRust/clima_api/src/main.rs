use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Deserialize, Serialize)] 
struct ClimateData {
    description: String,
    country: String,
    weather: String,
    // weather: WeatherType,
}

// #[derive(Debug, Deserialize, Serialize)] 
// #[serde(rename_all = "lowercase")]  //Debe mandarse como minuscula/ usar si solo se desea estos tipos en especifico los del enum
// enum WeatherType {
//     Rainy,
//     Cloudy,
//     Sunny,
// }

#[get("/")]
async fn welcome() -> impl Responder {
    HttpResponse::Ok().body("Welcome to my API in Rust / SOPES1_2025.")
}

#[post("/input")]
async fn receive_data(data: web::Json<ClimateData>) -> impl Responder {
    println!("Received: {:?}", data);
    // println!("Received: Description {}, Country {}", data.description, data.country);
    // HttpResponse::Ok().body("Received climate data.")  //mensaje simple

    let client = reqwest::Client::new();
    let result = client
        // .post("http://localhost:8081/input")
        .post("http://go-api-service:8081/input")
        .json(&*data)
        .send()
        .await;

    match result {
        Ok(resp) => {
            println!("Reenviado a Go. Status: {}", resp.status());
            HttpResponse::Ok().body("Datos reenviados al servicio Go")
        }
        Err(err) => {
            eprintln!("❌ Error al reenviar a Go: {}", err);
            HttpResponse::InternalServerError().body("Error al reenviar al servicio Go")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // println!("API RUST running at http://localhost:8080");
    println!("API RUST running at http://rust-api-service:8080");
    HttpServer::new(|| {
        App::new()
            .service(welcome)
            .service(receive_data)
    })
    // .bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
