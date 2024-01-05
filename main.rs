#![allow(non_snake_case)]
use serde_derive::{Deserialize, Serialize};
use std::env;
use tokio_postgres::{Client, NoTls};
use warp::{http::StatusCode, Filter};

#[derive(Deserialize, Serialize, Debug)]
struct Measurement {
    lpg: i32,
    lng: i32,
    co: i32,
    #[serde(default)]
    dateTime: String,
}

#[derive(Deserialize, Serialize)]
struct ListMeasurementsResponse {
    measurements: Vec<Measurement>,
}

async fn storeMeasurement(msmt: Measurement) -> Result<Measurement, tokio_postgres::Error> {
    let client = connectToDb().await?;

    let rows = client
        .query(
            "INSERT INTO misc_measurements(lpg, lng, co)
             VALUES($1, $2, $3) 
             RETURNING lng, lpg, co, datetime::TEXT",
            &[&msmt.lpg, &msmt.lng, &msmt.co],
        )
        .await?;

    // We only expect one row to be returned from insert
    let row = rows.get(0).unwrap();
    let msmt = Measurement {
        lpg: row.get::<_, i32>("lpg").to_owned(),
        lng: row.get::<_, i32>("lng").to_owned(),
        co: row.get::<_, i32>("co").to_owned(),
        dateTime: row.get::<_, String>("datetime").to_owned(),
    };
    Ok(msmt)
}

async fn listMeasurements() -> Result<Vec<Measurement>, tokio_postgres::Error> {
    let client = connectToDb().await?;

    let rows = client
        .query(
            "SELECT id, lpg, lng, co, datetime::TEXT AS datetime
            FROM misc_measurements
            ORDER BY datetime DESC",
            &[],
        )
        .await?;

    let mut results: Vec<Measurement> = Vec::new();
    for row in rows {
        results.push(Measurement {
            lpg: row.get::<_, i32>("lpg").to_owned(),
            lng: row.get::<_, i32>("lng").to_owned(),
            co: row.get::<_, i32>("co").to_owned(),
            dateTime: row.get::<_, String>("datetime").to_owned(),
        });
    }
    Ok(results)
}

async fn connectToDb() -> Result<Client, tokio_postgres::Error> {
    let host = env::var("AUTOCLEARSKIES_DB_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("AUTOCLEARSKIES_DB_PORT").unwrap_or_else(|_| "1337".to_string());
    let user = env::var("AUTOCLEARSKIES_DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let pass = env::var("AUTOCLEARSKIES_DB_PASS").unwrap_or_else(|_| "blab".to_string());

    let (client, connection) = tokio_postgres::connect(
        &format!("host={host} user={user} port={port} password={pass} dbname=autoclearskiesdb"),
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

async fn handleStoreMsmtReq(msmt: Measurement) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let r = storeMeasurement(msmt).await;
    if r.is_ok() {
        let result = r.unwrap();
        println!("insertion date received from db: {:?}", result.dateTime);
        Ok(Box::new(warp::reply::json(&result)))
    } else {
        eprintln!("{}", r.unwrap_err());
        Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

async fn handleListMsmtReq() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let r = listMeasurements().await;
    if r.is_ok() {
        let results = r.unwrap();
        Ok(Box::new(warp::reply::json(&ListMeasurementsResponse {
            measurements: results,
        })))
    } else {
        eprintln!("{}", r.unwrap_err());
        Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

#[tokio::main]
async fn main() {
    let host = env::var("AUTOCLEARSKIES_DB_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("AUTOCLEARSKIES_DB_PORT").unwrap_or_else(|_| "1337".to_string());
    let user = env::var("AUTOCLEARSKIES_DB_USER").unwrap_or_else(|_| "postgres".to_string());
    println!("Using db params host={host} port={port} user={user}");

    // POST /measurements/record with JSON body => 200 OK with JSON body
    let storeMsmtRoute = warp::post()
        .and(warp::path("measurements"))
        .and(warp::path("record"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(handleStoreMsmtReq);

    // GET /measurements => 200 OK with JSON body
    let listMsmtRoute = warp::get()
        .and(warp::path("measurements"))
        .and_then(handleListMsmtReq);

    println!("Starting server...");
    warp::serve(storeMsmtRoute.or(listMsmtRoute))
        .run(([0, 0, 0, 0], 3030))
        .await;
}
