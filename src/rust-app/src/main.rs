async fn docs() -> HttpResponse {
    let html = r#"
        <html>
        <head><title>Weather API Docs</title></head>
        <body>
            <h1>Weather API Documentation</h1>
            <ul>
                <li><strong>GET /countries</strong> - List all available countries</li>
                <li><strong>GET /countries/&lt;country&gt;</strong> - List all cities in a country</li>
                <li><strong>GET /countries/&lt;country&gt;/&lt;city&gt;/&lt;month&gt;</strong> - Get monthly weather averages</li>
            </ul>
        </body>
        </html>
    "#;
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use actix_web::{body::to_bytes, test, web, App};

    fn sample_data() -> WeatherData {
        let data = json!({
            "England": {
                "London": {
                    "January": {"high": 45, "low": 36},
                    "February": {"high": 46, "low": 36}
                },
                "Manchester": {
                    "January": {"high": 42, "low": 34}
                }
            },
            "France": {
                "Paris": {
                    "January": {"high": 45, "low": 36}
                }
            }
        });
        Arc::new(data)
    }

    #[actix_web::test]
    async fn test_countries() {
        let app = test::init_service(
            App::new().app_data(web::Data::new(sample_data())).route("/countries", web::get().to(countries))
        ).await;
        let req = test::TestRequest::get().uri("/countries").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        let countries: Vec<String> = serde_json::from_slice(&body).unwrap();
        assert!(countries.contains(&"England".to_string()));
        assert!(countries.contains(&"France".to_string()));
    }

    #[actix_web::test]
    async fn test_cities_england() {
        let app = test::init_service(
            App::new().app_data(web::Data::new(sample_data())).route("/countries/{country}", web::get().to(cities))
        ).await;
        let req = test::TestRequest::get().uri("/countries/England").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        let cities: Vec<String> = serde_json::from_slice(&body).unwrap();
        assert!(cities.contains(&"London".to_string()));
        assert!(cities.contains(&"Manchester".to_string()));
    }

    #[actix_web::test]
    async fn test_monthly_average_london_january() {
        let app = test::init_service(
            App::new().app_data(web::Data::new(sample_data())).route("/countries/{country}/{city}/{month}", web::get().to(monthly_average))
        ).await;
        let req = test::TestRequest::get().uri("/countries/England/London/January").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        let weather: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(weather["high"], 45);
        assert_eq!(weather["low"], 36);
    }

    #[actix_web::test]
    async fn test_cities_not_found() {
        let app = test::init_service(
            App::new().app_data(web::Data::new(sample_data())).route("/countries/{country}", web::get().to(cities))
        ).await;
        let req = test::TestRequest::get().uri("/countries/Unknown").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }
}
use actix_web::{web, App, HttpServer, HttpResponse, middleware, http::header};
use serde_json::{json, Value};
use std::fs;
use std::sync::Arc;

type WeatherData = Arc<Value>;

async fn root() -> HttpResponse {
    HttpResponse::Found()
        .insert_header((header::LOCATION, "/docs"))
        .finish()
}

async fn countries(data: web::Data<WeatherData>) -> HttpResponse {
    if let Some(obj) = data.as_object() {
        let countries: Vec<String> = obj.keys().cloned().collect();
        HttpResponse::Ok().json(countries)
    } else {
        HttpResponse::InternalServerError().json(json!({"error": "Invalid data"}))
    }
}


async fn cities(
    data: web::Data<WeatherData>,
    path: web::Path<String>,
) -> HttpResponse {
    let country_param = path.into_inner();
    if let Some(obj) = data.as_object() {
        // Try exact match first
        if let Some(country_data) = obj.get(&country_param) {
            if let Some(city_obj) = country_data.as_object() {
                let cities: Vec<String> = city_obj.keys().cloned().collect();
                return HttpResponse::Ok().json(cities);
            }
        }
        // Try case-insensitive match if exact fails
        if let Some((_, country_data)) = obj.iter().find(|(k, _)| k.eq_ignore_ascii_case(&country_param)) {
            if let Some(city_obj) = country_data.as_object() {
                let cities: Vec<String> = city_obj.keys().cloned().collect();
                return HttpResponse::Ok().json(cities);
            }
        }
    }
    HttpResponse::NotFound().json(json!({"error": "Country not found"}))
}

async fn monthly_average(
    data: web::Data<WeatherData>,
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    let (country, city, month) = path.into_inner();
    if let Some(country_data) = data.get(&country) {
        if let Some(city_data) = country_data.get(&city) {
            if let Some(month_data) = city_data.get(&month) {
                return HttpResponse::Ok().json(month_data);
            }
        }
    }
    HttpResponse::NotFound().json(json!({"error": "Not found"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load weather data from JSON file
    // Try multiple paths: current dir, parent dir, and absolute path
    let weather_file = [
        "weather.json".to_string(),
        "../weather.json".to_string(),
        "/workspaces/aitour26-WRK541-real-world-code-migration-with-github-copilot-agent-mode/src/rust-app/weather.json".to_string(),
    ].iter()
        .find_map(|path| {
            fs::read_to_string(path).ok().map(|_| path.clone())
        })
        .expect("Failed to find weather.json in any expected location");
    
    let weather_json = fs::read_to_string(&weather_file)
        .expect("Failed to read weather.json");
    let weather_data: Value = serde_json::from_str(&weather_json)
        .expect("Failed to parse weather.json");
    let weather_data = Arc::new(weather_data);
    
    println!("Starting Weather API server on http://127.0.0.1:8000");
    
    let weather_data_clone = weather_data.clone();
    
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(weather_data_clone.clone()))
            .route("/", web::get().to(root))
            .route("/docs", web::get().to(docs))
            .route("/countries", web::get().to(countries))
            .route("/countries/{country}", web::get().to(cities))
            .route("/countries/{country}/{city}/{month}", web::get().to(monthly_average))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
