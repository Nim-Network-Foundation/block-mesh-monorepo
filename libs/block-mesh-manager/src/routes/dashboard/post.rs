use crate::errors::error::Error;
use crate::middlewares::authentication::Backend;
use crate::routes::dashboard::dashboard_data_extractor::dashboard_data_extractor;
use crate::startup::application::AppState;
use axum::extract::State;
use axum::{Extension, Json};
use axum_login::AuthSession;
use block_mesh_common::interfaces::db_messages::{AggregateAddToMessage, DBMessageTypes};
use block_mesh_common::interfaces::server_api::DashboardResponse;
use block_mesh_manager_database_domain::domain::aggregate::AggregateName;
use block_mesh_manager_database_domain::domain::create_daily_stat::get_or_create_daily_stat;
use block_mesh_manager_database_domain::domain::notify_worker::notify_worker;
use chrono::{Duration, Utc};
use database_utils::utils::instrument_wrapper::{commit_txn, create_txn};
use sqlx::PgPool;
use std::sync::Arc;
#[allow(unused_imports)]
use tracing::Level;

#[tracing::instrument(name = "dashboard", skip_all)]
pub async fn handler(
    Extension(pool): Extension<PgPool>,
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthSession<Backend>>,
) -> Result<Json<DashboardResponse>, Error> {
    let user = auth.user.ok_or(Error::UserNotFound)?;
    let _ = notify_worker(
        &pool,
        AggregateAddToMessage {
            msg_type: DBMessageTypes::AggregateAddToMessage,
            user_id: user.id,
            value: serde_json::Value::from(1),
            name: AggregateName::Uptime.to_string(),
        },
    )
    .await;
    let mut transaction = create_txn(&pool).await?;
    let today = Utc::now().date_naive();
    for i in 0..14 {
        let day = today - Duration::days(i);
        let _ = get_or_create_daily_stat(&mut transaction, &user.id, Some(day)).await?;
    }
    commit_txn(transaction).await?;
    let data = dashboard_data_extractor(&pool, user.id, state).await?;
    Ok(Json(data))
}
