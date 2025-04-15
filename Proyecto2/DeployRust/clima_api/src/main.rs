use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ClimateData {
    description: String,
    country: String,
    weather: String,
    // weather: WeatherType,
}

// #[derive(Debug, Deserialize)]
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
    HttpResponse::Ok().body("Received climate data.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 API running at http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .service(welcome)
            .service(receive_data)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
