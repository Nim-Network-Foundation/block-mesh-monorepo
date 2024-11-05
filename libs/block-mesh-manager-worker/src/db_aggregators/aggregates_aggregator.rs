use anyhow::anyhow;
use block_mesh_common::interfaces::db_messages::AggregateMessage;
use chrono::Utc;
use database_utils::utils::instrument_wrapper::{commit_txn, create_txn};
use flume::Sender;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[tracing::instrument(name = "aggregates_create_bulk_query", skip_all)]
pub fn aggregates_create_bulk_query(calls: HashMap<Uuid, Value>) -> String {
    let now = Utc::now();
    let values: Vec<String> = calls
        .iter()
        .map(|(id, value)| {
            format!(
                "('{}'::uuid, '{}'::jsonb, '{}'::timestamptz)",
                id,
                value,
                now.to_rfc3339()
            )
        })
        .collect();

    let value_str = values.join(",");
    format!(
        r#"
-- Temporary table or CTE holding new values
WITH updates (id, value, updated_at) AS (
VALUES {}
)
-- Update statement using the CTE
UPDATE aggregates
SET
    value = updates.value,
    updated_at = updates.updated_at
FROM updates
WHERE aggregates.id = updates.id;
"#,
        value_str
    )
}

#[tracing::instrument(name = "aggregates_aggregator", skip_all, err)]
pub async fn aggregates_aggregator(
    joiner_tx: Sender<JoinHandle<()>>,
    pool: PgPool,
    mut rx: Receiver<Value>,
    agg_size: i32,
    time_limit: i64,
) -> Result<(), anyhow::Error> {
    let mut calls: HashMap<_, _> = HashMap::new();
    let mut count = 0;
    let mut prev = Utc::now();
    loop {
        match rx.recv().await {
            Ok(message) => {
                if let Ok(message) = serde_json::from_value::<AggregateMessage>(message) {
                    calls.insert(message.id, message.value);
                    count += 1;
                    let now = Utc::now();
                    let diff = now - prev;
                    let run = diff.num_seconds() > time_limit || count >= agg_size;
                    prev = Utc::now();
                    if run {
                        let calls_clone = calls.clone();
                        let poll_clone = pool.clone();
                        let handle = tokio::spawn(async move {
                            if let Ok(mut transaction) = create_txn(&poll_clone).await {
                                let query = aggregates_create_bulk_query(calls_clone);
                                let _ = sqlx::query(&query)
                                    .execute(&mut *transaction)
                                    .await
                                    .map_err(|e| {
                                        tracing::error!(
                                            "Failed to execute query: {} , with error {:?}",
                                            query,
                                            e
                                        );
                                    });
                                let _ = commit_txn(transaction).await;
                            }
                        });
                        let _ = joiner_tx.send_async(handle).await;
                        count = 0;
                        calls.clear();
                    }
                }
            }
            Err(e) => match e {
                RecvError::Closed => {
                    tracing::error!("aggregates_aggregator error recv: {:?}", e);
                    return Err(anyhow!("aggregates_aggregator error recv: {:?}", e));
                }
                RecvError::Lagged(_) => {
                    tracing::error!("aggregates_aggregator error recv: {:?}", e);
                }
            },
        }
    }
}
