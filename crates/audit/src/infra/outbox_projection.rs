use audit_contract::{AUDIT_OUTBOX_PAYLOAD_VERSION, AuditOutboxEvent};
use sqlx::{FromRow, Postgres, Transaction, query, query_as};

use crate::{
    application::{AuditError, AuditResult},
    domain::{AuditLocation, NewLoginLog, NewOperationLog, OperationLogDetail, OperationLogSummary},
};

use super::{command, outbox_repository::ClaimedAuditEvent};

#[derive(FromRow)]
struct ProjectionRow {
    stream: String,
    event_type: String,
    payload_version: i16,
    payload: serde_json::Value,
}

pub(crate) async fn project_and_mark(transaction: &mut Transaction<'_, Postgres>, claimed: &ClaimedAuditEvent, location: AuditLocation) -> AuditResult<bool> {
    let Some(row) = query_as::<_, ProjectionRow>(
        "SELECT stream,event_type,payload_version,payload FROM audit_outbox WHERE outbox_id=$1 AND lease_token=$2 AND processed_at IS NULL FOR UPDATE",
    )
    .bind(&claimed.id)
    .bind(&claimed.lease_token)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(super::mapping::sqlx_error)?
    else {
        return Ok(false);
    };
    let event = decode_event(row)?;
    insert_projection(transaction, InsertProjectionCommand { claimed, event, location }).await?;
    query(
        "UPDATE audit_outbox SET processed_at=CURRENT_TIMESTAMP, lease_token=NULL, lease_until=NULL, last_error_code=NULL WHERE outbox_id=$1 AND lease_token=$2 AND processed_at IS NULL",
    )
    .bind(&claimed.id)
    .bind(&claimed.lease_token)
    .execute(&mut **transaction)
    .await
    .map_err(super::mapping::sqlx_error)?;
    Ok(true)
}

fn decode_event(row: ProjectionRow) -> AuditResult<AuditOutboxEvent> {
    if row.payload_version != AUDIT_OUTBOX_PAYLOAD_VERSION {
        return Err(AuditError::Infrastructure(format!(
            "unsupported audit outbox payload version: {}",
            row.payload_version
        )));
    }
    let event = serde_json::from_value::<AuditOutboxEvent>(row.payload).map_err(|error| AuditError::Infrastructure(error.to_string()))?;
    if event.stream().code() != row.stream {
        return Err(AuditError::Infrastructure("audit outbox stream does not match payload kind".into()));
    }
    if event.event_type() != row.event_type {
        return Err(AuditError::Infrastructure("audit outbox event type does not match payload".into()));
    }
    Ok(event)
}

struct InsertProjectionCommand<'a> {
    claimed: &'a ClaimedAuditEvent,
    event: AuditOutboxEvent,
    location: AuditLocation,
}

async fn insert_projection(transaction: &mut Transaction<'_, Postgres>, input: InsertProjectionCommand<'_>) -> AuditResult<()> {
    match input.event {
        AuditOutboxEvent::Operation(event) => {
            command::insert_operation_in_transaction(transaction, operation_log(input.claimed, event, input.location), true).await
        }
        AuditOutboxEvent::Security(event) => {
            let insert = command::LoginInsertCommand::idempotent(input.claimed.id.clone(), login_log(input.claimed, event, input.location));
            command::insert_login_in_transaction(transaction, insert).await
        }
    }
}

fn operation_log(claimed: &ClaimedAuditEvent, event: audit_contract::OperationAuditEvent, location: AuditLocation) -> NewOperationLog {
    NewOperationLog {
        detail: OperationLogDetail {
            summary: OperationLogSummary {
                id: claimed.id.clone(),
                title_key: event.title_key,
                business_type: event.business_type,
                handler: event.handler,
                request_method: event.request_method,
                operator_type: event.operator_type,
                operator_name: event.actor.username,
                department_name: event.actor.department_name,
                operation_url: event.operation_url,
                operation_ip: event.operation_ip,
                operation_location: location,
                status: event.status,
                operation_time: claimed.occurred_at,
                cost_time_ms: event.cost_time_ms,
            },
            request_id: event.request_id,
            operator_id: event.actor.user_id,
            department_id: event.actor.department_id,
            request_params: event.request_params,
            response_result: event.response_result,
            error_message: event.error_message,
        },
    }
}

fn login_log(claimed: &ClaimedAuditEvent, event: audit_contract::SecurityAuditEvent, location: AuditLocation) -> NewLoginLog {
    NewLoginLog {
        request_id: event.request_id,
        route: event.route,
        user_id: event.user_id,
        username: event.username,
        ip_address: event.ip_address,
        login_location: location,
        browser: event.browser,
        os: event.os,
        status: event.status,
        event_type: event.event_type,
        message_key: event.message_key,
        message_params: event.message_params,
        login_time: claimed.occurred_at,
    }
}
