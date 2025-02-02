use rand::Rng;
use serde_json::Value;
use sql_client::models::Submission;
use sql_client::PgPool;

pub mod utils;

async fn prepare_data_set(conn: &PgPool) {
    sql_client::query(r"INSERT INTO accepted_count (user_id, problem_count) VALUES ('u1', 1)")
        .execute(conn)
        .await
        .unwrap();
    sql_client::query(r"INSERT INTO rated_point_sum (user_id, point_sum) VALUES ('u1', 1.0)")
        .execute(conn)
        .await
        .unwrap();
    sql_client::query(
        r"
    INSERT INTO
        submissions (epoch_second, problem_id, contest_id, user_id, result, id, language, point, length)
        VALUES
            (0,  'p1',   'c1',   'u1',   'WA',   1,  'Rust',    0.0,    0),
            (1,  'p1',   'c1',   'u1',   'RE',   2,  'Rust',    0.0,    0),
            (2,  'p1',   'c1',   'u1',   'AC',   3,  'Rust',    0.0,    0),
            (3,  'p1',   'c1',   'u1',   'AC',   4,  'Rust',    0.0,    0),
            (100,'p1',   'c1',   'u1',   'AC',   5,  'Rust',    0.0,    0),
            (4,  'p1',   'c1',   'u2',   'WA',   6,  'Rust',    0.0,    0),
            (5,  'p1',   'c1',   'u2',   'RE',   7,  'Rust',    0.0,    0),
            (6,  'p1',   'c1',   'u2',   'AC',   8,  'Rust',    0.0,    0),
            (7,  'p1',   'c1',   'u2',   'AC',   9,  'Rust',    0.0,    0),
            (200,'p1',   'c1',   'u2',   'AC',   10, 'Rust',    0.0,    0)",
    )
    .execute(conn)
    .await
    .unwrap();
}

fn url(path: &str, port: u16) -> String {
    format!("http://localhost:{}{}", port, path)
}

async fn setup() -> u16 {
    prepare_data_set(&utils::initialize_and_connect_to_test_sql().await).await;
    let mut rng = rand::thread_rng();
    rng.gen::<u16>() % 30000 + 30000
}

#[actix_web::test]
async fn test_user_submissions() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    let submissions: Vec<Submission> = reqwest::get(url("/atcoder-api/results?user=u1", port))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(submissions.len(), 5);
    assert!(submissions.iter().all(|s| s.user_id.as_str() == "u1"));

    let response = reqwest::get(url("/atcoder-api/results?user=u2", port))
        .await
        .unwrap();
    let submissions: Vec<Submission> = response.json().await.unwrap();
    assert_eq!(submissions.len(), 5);
    assert!(submissions.iter().all(|s| s.user_id.as_str() == "u2"));

    server.abort();
    server.await.unwrap_err();
}

#[actix_web::test]
async fn test_user_submissions_fromtime() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    let submissions: Vec<Submission> = reqwest::get(url(
        "/atcoder-api/v3/user/submissions?user=u1&from_second=3",
        port,
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(submissions.len(), 2);
    assert!(submissions.iter().all(|s| s.user_id.as_str() == "u1"));

    let response = reqwest::get(url(
        "/atcoder-api/v3/user/submissions?user=u2&from_second=6",
        port,
    ))
    .await
    .unwrap();
    let submissions: Vec<Submission> = response.json().await.unwrap();
    assert_eq!(submissions.len(), 3);
    assert!(submissions.iter().all(|s| s.user_id.as_str() == "u2"));
    assert_eq!(submissions[0].epoch_second, 6);
    assert_eq!(submissions[1].epoch_second, 7);
    assert_eq!(submissions[2].epoch_second, 200);

    let response = reqwest::get(url(
        "/atcoder-api/v3/user/submissions?user=u3&from_second=0",
        port,
    ))
    .await
    .unwrap();
    let submissions: Vec<Submission> = response.json().await.unwrap();
    assert_eq!(submissions.len(), 0);

    let response = reqwest::get(url(
        "/atcoder-api/v3/user/submissions?user=u1&from_second=-30",
        port,
    ))
    .await
    .unwrap();
    let submissions: Vec<Submission> = response.json().await.unwrap();
    assert_eq!(submissions.len(), 5);

    let response = reqwest::get(url(
        "/atcoder-api/v3/user/submissions?user=u2&from_second=3000",
        port,
    ))
    .await
    .unwrap();
    let submissions: Vec<Submission> = response.json().await.unwrap();
    assert_eq!(submissions.len(), 0);

    server.abort();
    server.await.unwrap_err();
}

#[actix_web::test]
async fn test_time_submissions() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    let submissions: Vec<Submission> = reqwest::get(url("/atcoder-api/v3/from/100", port))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(submissions.len(), 2);
    assert!(submissions.iter().all(|s| s.epoch_second >= 100));

    server.abort();
    server.await.unwrap_err();
}

#[actix_web::test]
async fn test_submission_count() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    let response: Value = reqwest::get(url(
        r"/atcoder-api/v3/user/submission_count?user=u1&from_second=1&to_second=4",
        port,
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(response["count"], serde_json::json!(3));
    let response: Value = reqwest::get(url(
        r"/atcoder-api/v3/user/submission_count?user=u1&from_second=1&to_second=3",
        port,
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(response["count"], serde_json::json!(2));

    server.abort();
    server.await.unwrap_err();
}

#[actix_web::test]
async fn test_invalid_path() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    let response = reqwest::get(url("/atcoder-api/v3/from/", port))
        .await
        .unwrap();
    assert_eq!(response.status(), 404);

    let response = reqwest::get(url("/atcoder-api/results", port))
        .await
        .unwrap();
    assert_eq!(response.status(), 400);

    let response = reqwest::get(url("/", port)).await.unwrap();
    assert_eq!(response.status(), 404);

    server.abort();
    server.await.unwrap_err();
}

#[actix_web::test]
async fn test_health_check() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    let response = reqwest::get(url("/healthcheck", port)).await.unwrap();
    assert_eq!(response.status(), 200);

    server.abort();
    server.await.unwrap_err();
}

#[actix_web::test]
async fn test_cors() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    assert_eq!(
        reqwest::get(url("/atcoder-api/v3/from/100", port))
            .await
            .unwrap()
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "*"
    );
    assert_eq!(
        reqwest::get(url("/atcoder-api/v2/user_info?user=u1", port))
            .await
            .unwrap()
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "*"
    );
    assert_eq!(
        reqwest::get(url("/atcoder-api/results?user=u1", port))
            .await
            .unwrap()
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "*"
    );

    server.abort();
    server.await.unwrap_err();
}

#[actix_web::test]
async fn test_users_and_time() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;
    let submissions: Vec<Submission> = reqwest::get(url(
        "/atcoder-api/v3/users_and_time?users=u1,u2&problems=p1&from=100&to=200",
        port,
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(submissions.len(), 2);
    assert_eq!(submissions.iter().filter(|s| &s.user_id == "u1").count(), 1);
    assert_eq!(submissions.iter().filter(|s| &s.user_id == "u2").count(), 1);

    server.abort();
    server.await.unwrap_err();
}
