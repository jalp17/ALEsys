use sqlx::PgPool;
use sqlx::Row;
use std::env;

#[tokio::test]
async fn ingestion_jobs_db_integration() {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL required for integration test");
    let pool = PgPool::connect(&db_url).await.expect("connect postgres");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS ingestion_jobs (
            id UUID PRIMARY KEY,
            pdf_path TEXT,
            topic TEXT,
            status TEXT,
            progress FLOAT,
            message TEXT,
            output_dir TEXT,
            markdown_path TEXT,
            created_at TIMESTAMPTZ
        )"#,
    )
    .execute(&pool)
    .await
    .expect("create table");

    let job_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO ingestion_jobs (id, pdf_path, topic, status, progress, message, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, NOW())"#,
    )
    .bind(job_id)
    .bind("/tmp/test.pdf")
    .bind("integration-test")
    .bind("queued")
    .bind(0.0)
    .bind("starting")
    .execute(&pool)
    .await
    .expect("insert job");

    let row = sqlx::query(
        "SELECT status, progress, topic FROM ingestion_jobs WHERE id = $1",
    )
    .bind(job_id)
    .fetch_one(&pool)
    .await
    .expect("select job");

    assert_eq!(row.get::<String, _>("status"), "queued");
    assert_eq!(row.get::<String, _>("topic"), "integration-test");
    assert!((row.get::<f64, _>("progress") - 0.0).abs() < f64::EPSILON);

    sqlx::query("UPDATE ingestion_jobs SET status = $1, progress = $2 WHERE id = $3")
        .bind("completed")
        .bind(1.0)
        .bind(job_id)
        .execute(&pool)
        .await
        .expect("update job");

    let row = sqlx::query(
        "SELECT status, progress FROM ingestion_jobs WHERE id = $1",
    )
    .bind(job_id)
    .fetch_one(&pool)
    .await
    .expect("select after update");

    assert_eq!(row.get::<String, _>("status"), "completed");

    sqlx::query("DELETE FROM ingestion_jobs WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await
        .expect("delete job");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingestion_jobs WHERE id = $1")
        .bind(job_id)
        .fetch_one(&pool)
        .await
        .expect("count after delete");

    assert_eq!(count, 0);
}
