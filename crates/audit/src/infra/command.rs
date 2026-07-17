use audit_contract::{AuditOutboxRecord, AuditStream};
use sqlx::{AssertSqlSafe, Postgres, Transaction, query, query_scalar};
use storage::ObservedPgPool;
use storage::outbox::{append_audit_record, clear_audit_stream};

use crate::{
    application::{AuditError, AuditResult},
    domain::{NewLoginLog, NewOperationLog},
};

use super::mapping;

pub async fn insert_operation(pool: ObservedPgPool, log: NewOperationLog) -> AuditResult<()> {
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    insert_operation_in_transaction(&mut transaction, log, false).await?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

pub async fn insert_operation_in_transaction(transaction: &mut Transaction<'_, Postgres>, log: NewOperationLog, ignore_duplicate: bool) -> AuditResult<()> {
    let detail = log.detail;
    let summary = detail.summary;
    let location_kind = summary.operation_location.kind();
    let location_text = summary.operation_location.text();
    query(operation_insert_sql(ignore_duplicate))
        .bind(summary.id)
        .bind(detail.request_id)
        .bind(summary.title_key)
        .bind(summary.business_type.code())
        .bind(summary.handler)
        .bind(summary.request_method)
        .bind(summary.operator_type.code())
        .bind(detail.operator_id)
        .bind(summary.operator_name)
        .bind(detail.department_id)
        .bind(summary.department_name)
        .bind(summary.operation_url)
        .bind(summary.operation_ip)
        .bind(location_kind)
        .bind(location_text)
        .bind(detail.request_params)
        .bind(detail.response_result)
        .bind(summary.status.code())
        .bind(detail.error_message)
        .bind(summary.operation_time)
        .bind(summary.cost_time_ms)
        .execute(&mut **transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    Ok(())
}

pub async fn insert_login(pool: ObservedPgPool, id: String, log: NewLoginLog) -> AuditResult<()> {
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    insert_login_in_transaction(&mut transaction, LoginInsertCommand::strict(id, log)).await?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

pub(super) struct LoginInsertCommand {
    id: String,
    log: NewLoginLog,
    ignore_duplicate: bool,
}

impl LoginInsertCommand {
    fn strict(id: String, log: NewLoginLog) -> Self {
        Self {
            id,
            log,
            ignore_duplicate: false,
        }
    }

    pub(super) fn idempotent(id: String, log: NewLoginLog) -> Self {
        Self {
            id,
            log,
            ignore_duplicate: true,
        }
    }
}

pub(super) async fn insert_login_in_transaction(transaction: &mut Transaction<'_, Postgres>, command: LoginInsertCommand) -> AuditResult<()> {
    let LoginInsertCommand { id, log, ignore_duplicate } = command;
    let params = serde_json::to_value(log.message_params).map_err(|error| AuditError::Infrastructure(error.to_string()))?;
    let location_kind = log.login_location.kind();
    let location_text = log.login_location.text();
    query(login_insert_sql(ignore_duplicate))
        .bind(id)
        .bind(log.user_id)
        .bind(log.username)
        .bind(log.ip_address)
        .bind(location_kind)
        .bind(location_text)
        .bind(log.browser)
        .bind(log.os)
        .bind(log.status.code())
        .bind(log.event_type.code())
        .bind(log.message_key)
        .bind(params)
        .bind(log.login_time)
        .execute(&mut **transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    Ok(())
}

pub async fn delete_operations(pool: ObservedPgPool, ids: &[String]) -> AuditResult<()> {
    exact_delete(
        pool,
        ExactDelete {
            table: "sys_oper_log",
            column: "oper_id",
            ids,
        },
    )
    .await
}

pub async fn delete_operations_with_audit(pool: ObservedPgPool, ids: &[String], record: &AuditOutboxRecord) -> AuditResult<()> {
    exact_delete_with_audit(
        pool,
        ExactDelete {
            table: "sys_oper_log",
            column: "oper_id",
            ids,
        },
        record,
    )
    .await
}

pub async fn delete_logins(pool: ObservedPgPool, ids: &[String]) -> AuditResult<()> {
    exact_delete(
        pool,
        ExactDelete {
            table: "sys_logininfor",
            column: "info_id",
            ids,
        },
    )
    .await
}

pub async fn delete_logins_with_audit(pool: ObservedPgPool, ids: &[String], record: &AuditOutboxRecord) -> AuditResult<()> {
    exact_delete_with_audit(
        pool,
        ExactDelete {
            table: "sys_logininfor",
            column: "info_id",
            ids,
        },
        record,
    )
    .await
}

pub async fn clear_operations(pool: ObservedPgPool) -> AuditResult<()> {
    clear_stream(pool, OPERATION_LOG_CLEAR).await
}

pub async fn clear_logins(pool: ObservedPgPool) -> AuditResult<()> {
    clear_stream(pool, LOGIN_LOG_CLEAR).await
}

pub async fn clear_operations_with_audit(pool: ObservedPgPool, record: &AuditOutboxRecord) -> AuditResult<()> {
    clear_stream_with_audit(pool, OPERATION_LOG_CLEAR, record).await
}

pub async fn clear_logins_with_audit(pool: ObservedPgPool, record: &AuditOutboxRecord) -> AuditResult<()> {
    clear_stream_with_audit(pool, LOGIN_LOG_CLEAR, record).await
}

fn operation_insert_sql(ignore_duplicate: bool) -> &'static str {
    match ignore_duplicate {
        true => {
            "INSERT INTO sys_oper_log (oper_id,request_id,title,business_type,method,request_method,operator_type,operator_id,oper_name,dept_id,dept_name,oper_url,oper_ip,oper_location_kind,oper_location,oper_param,json_result,status,error_msg,oper_time,cost_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21) ON CONFLICT (oper_id) DO NOTHING"
        }
        false => {
            "INSERT INTO sys_oper_log (oper_id,request_id,title,business_type,method,request_method,operator_type,operator_id,oper_name,dept_id,dept_name,oper_url,oper_ip,oper_location_kind,oper_location,oper_param,json_result,status,error_msg,oper_time,cost_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)"
        }
    }
}

fn login_insert_sql(ignore_duplicate: bool) -> &'static str {
    match ignore_duplicate {
        true => {
            "INSERT INTO sys_logininfor (info_id,user_id,user_name,ipaddr,login_location_kind,login_location,browser,os,status,event_type,message_key,message_params,login_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) ON CONFLICT (info_id) DO NOTHING"
        }
        false => {
            "INSERT INTO sys_logininfor (info_id,user_id,user_name,ipaddr,login_location_kind,login_location,browser,os,status,event_type,message_key,message_params,login_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"
        }
    }
}

#[derive(Clone, Copy)]
struct StreamClearCommand {
    stream: AuditStream,
    sql: &'static str,
}

const OPERATION_LOG_CLEAR: StreamClearCommand = StreamClearCommand {
    stream: AuditStream::Operation,
    sql: "TRUNCATE TABLE sys_oper_log",
};
const LOGIN_LOG_CLEAR: StreamClearCommand = StreamClearCommand {
    stream: AuditStream::Security,
    sql: "TRUNCATE TABLE sys_logininfor",
};

async fn clear_stream(pool: ObservedPgPool, command: StreamClearCommand) -> AuditResult<()> {
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    clear_audit_stream(&mut transaction, command.stream).await.map_err(storage_error)?;
    query(command.sql).execute(&mut *transaction).await.map_err(mapping::sqlx_error)?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

async fn clear_stream_with_audit(pool: ObservedPgPool, command: StreamClearCommand, record: &AuditOutboxRecord) -> AuditResult<()> {
    if record.stream() != AuditStream::Operation {
        return Err(AuditError::Infrastructure("audit log clear requires an operation outbox event".into()));
    }
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    clear_audit_stream(&mut transaction, command.stream).await.map_err(storage_error)?;
    query(command.sql).execute(&mut *transaction).await.map_err(mapping::sqlx_error)?;
    append_audit_record(&mut transaction, record).await.map_err(storage_error)?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

fn storage_error(error: storage::StorageError) -> AuditError {
    AuditError::Infrastructure(error.to_string())
}

#[derive(Clone, Copy)]
struct ExactDelete<'a> {
    table: &'static str,
    column: &'static str,
    ids: &'a [String],
}

async fn exact_delete(pool: ObservedPgPool, target: ExactDelete<'_>) -> AuditResult<()> {
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    ensure_all_exist(&mut transaction, target).await?;
    delete_rows(&mut transaction, target).await?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

async fn exact_delete_with_audit(pool: ObservedPgPool, target: ExactDelete<'_>, record: &AuditOutboxRecord) -> AuditResult<()> {
    if record.stream() != AuditStream::Operation {
        return Err(AuditError::Infrastructure("audit log deletion requires an operation outbox event".into()));
    }
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    ensure_all_exist(&mut transaction, target).await?;
    delete_rows(&mut transaction, target).await?;
    append_audit_record(&mut transaction, record).await.map_err(storage_error)?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

async fn delete_rows(transaction: &mut Transaction<'_, Postgres>, target: ExactDelete<'_>) -> AuditResult<()> {
    let sql = format!("DELETE FROM {} WHERE {}=ANY($1)", target.table, target.column);
    query(AssertSqlSafe(sql))
        .bind(target.ids)
        .execute(&mut **transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    Ok(())
}

async fn ensure_all_exist(transaction: &mut Transaction<'_, Postgres>, target: ExactDelete<'_>) -> AuditResult<()> {
    let sql = format!("SELECT {0} FROM {1} WHERE {0}=ANY($1) ORDER BY {0} FOR UPDATE", target.column, target.table);
    let existing = query_scalar::<_, String>(AssertSqlSafe(sql))
        .bind(target.ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    if existing.len() != target.ids.len() {
        return Err(AuditError::NotFound);
    }
    Ok(())
}
