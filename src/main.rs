//! Web service API for estimating Pi.
#[macro_use]
extern crate rocket;

use rand::random;
use rocket::http::CookieJar;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::fs::FileServer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MonteCarloResult {
    inside: u64,
    total: u64,
    pi: f64,
}

#[get("/monte_carlo?<n>")]
fn monte_carlo(cookies: &CookieJar, n: Option<u64>) -> Json<MonteCarloResult> {
    let total = cookies
        .get("n")
        .map(|c| c.value())
        .map(|v| v.parse().ok())
        .unwrap_or(n)
        .unwrap_or(100_000);

    let mut inside = 0;
    for _ in 0..total {
        let x = random::<f64>();
        let y = random::<f64>();
        if x * x + y * y <= 1.0 {
            inside += 1;
        }
    }

    Json(MonteCarloResult {
        inside,
        total,
        pi: 4.0 * inside as f64 / total as f64,
    })
}

#[post("/defaults?<n>")]
fn defaults(cookies: &CookieJar, n: Option<u64>) -> Status {
    if let Some(n) = n {
        cookies.add(("n", n.to_string()));
    } else {
        cookies.remove("n");
    }
    Status::Ok
}

#[launch]
fn app() -> _ {
    rocket::build()
        .mount("/pi", routes![monte_carlo, defaults])
        .mount("/", FileServer::from("static"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn test_monte_carlo() {
        let n = 1000;
        let client = Client::tracked(app()).unwrap();
        let response = client.get(format!("/pi/monte_carlo?n={n}")).dispatch();

        assert_eq!(response.status(), Status::Ok);

        let result: MonteCarloResult = response.into_json().unwrap();
        assert_eq!(result.total, n);
        assert_relative_eq!(result.pi, std::f64::consts::PI, epsilon = 1.0);
    }

    // TODO: add more tests for defaults
}
