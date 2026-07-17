mod keyset;

use sqlx::{AssertSqlSafe, PgConnection, PgPool, Postgres, QueryBuilder, query_as};
use time::OffsetDateTime;

use crate::{
    application::{ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportSlice, SystemLogSnapshot},
    domain::{SystemLogDetail, SystemLogFilter},
};

use super::{
    mapping,
    records::{SystemLogDetailRecord, SystemLogSummaryRecord},
};

const SUMMARY_COLUMNS: &str = "id,occurred_at,level,target,message";
const DETAIL_COLUMNS: &str = "id,occurred_at,level,target,message,fields";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchStrategy {
    FullText,
    Trigram,
    UnicodeNgram,
}

pub(super) async fn page(pool: &PgPool, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice> {
    let mut connection = pool.acquire().await.map_err(mapping::sqlx_error)?;
    page_on(&mut connection, filter, page).await
}

pub(super) async fn page_on(connection: &mut PgConnection, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice> {
    let Some(snapshot) = resolve_snapshot(connection, page.snapshot.clone()).await? else {
        return Ok(empty_slice());
    };
    let mut builder = QueryBuilder::<Postgres>::new(format!("SELECT {SUMMARY_COLUMNS} FROM sys_system_log WHERE TRUE"));
    push_filters(&mut builder, &filter);
    keyset::push_snapshot(&mut builder, &snapshot)?;
    keyset::push_boundary(&mut builder, &page)?;
    keyset::push_order(&mut builder, page.direction);
    keyset::push_limit(&mut builder, page.limit)?;
    let records = builder
        .build_query_as::<SystemLogSummaryRecord>()
        .fetch_all(&mut *connection)
        .await
        .map_err(mapping::sqlx_error)?;
    let items = records.into_iter().map(mapping::summary).collect::<ObservabilityResult<Vec<_>>>()?;
    keyset::slice(items, snapshot, page)
}

pub(super) async fn page_for_export_on(
    connection: &mut PgConnection,
    filter: SystemLogFilter,
    page: SystemLogCursorQuery,
) -> ObservabilityResult<SystemLogExportSlice> {
    let Some(snapshot) = resolve_snapshot(connection, page.snapshot.clone()).await? else {
        return Ok(empty_export_slice());
    };
    let mut builder = QueryBuilder::<Postgres>::new(format!("SELECT {DETAIL_COLUMNS} FROM sys_system_log WHERE TRUE"));
    push_filters(&mut builder, &filter);
    keyset::push_snapshot(&mut builder, &snapshot)?;
    keyset::push_boundary(&mut builder, &page)?;
    keyset::push_order(&mut builder, page.direction);
    keyset::push_limit(&mut builder, page.limit)?;
    let records = builder
        .build_query_as::<SystemLogDetailRecord>()
        .fetch_all(&mut *connection)
        .await
        .map_err(mapping::sqlx_error)?;
    let items = records.into_iter().map(mapping::detail).collect::<ObservabilityResult<Vec<_>>>()?;
    keyset::slice_export(items, snapshot, page)
}

pub(super) async fn find(pool: &PgPool, id: &str) -> ObservabilityResult<Option<SystemLogDetail>> {
    let sql = format!("SELECT {DETAIL_COLUMNS} FROM sys_system_log WHERE id=$1 ORDER BY occurred_at DESC,id DESC LIMIT 1");
    query_as::<_, SystemLogDetailRecord>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(mapping::sqlx_error)?
        .map(mapping::detail)
        .transpose()
}

pub(super) async fn count(pool: &PgPool, filter: SystemLogFilter) -> ObservabilityResult<u64> {
    let mut builder = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM sys_system_log WHERE TRUE");
    push_filters(&mut builder, &filter);
    let count = builder.build_query_scalar::<i64>().fetch_one(pool).await.map_err(mapping::sqlx_error)?;
    u64::try_from(count).map_err(|error| crate::application::ObservabilityError::Infrastructure(format!("system log count conversion failed: {error}")))
}

pub(super) fn push_filters(builder: &mut QueryBuilder<Postgres>, filter: &SystemLogFilter) {
    push_keyword(builder, filter.keyword.as_deref());
    push_levels(builder, filter);
    if let Some(target) = filter.target.as_deref() {
        builder
            .push(" AND (target=")
            .push_bind(target)
            .push(" OR target LIKE ")
            .push_bind(target_prefix_pattern(target))
            .push(" ESCAPE '\\')");
    }
    push_time_range(builder, filter.begin_time, filter.end_time);
}

async fn resolve_snapshot(connection: &mut PgConnection, snapshot: Option<SystemLogSnapshot>) -> ObservabilityResult<Option<SystemLogSnapshot>> {
    if let Some(snapshot) = snapshot {
        if snapshot.ingested_seq <= 0 {
            return Err(crate::application::ObservabilityError::InvalidCursor);
        }
        return Ok(Some(snapshot));
    }
    query_as::<_, (i64,)>("SELECT ingested_seq FROM sys_system_log ORDER BY ingested_seq DESC LIMIT 1")
        .fetch_optional(&mut *connection)
        .await
        .map_err(mapping::sqlx_error)
        .map(|row| row.map(|(ingested_seq,)| SystemLogSnapshot::new(ingested_seq)))
}

fn push_keyword(builder: &mut QueryBuilder<Postgres>, keyword: Option<&str>) {
    let Some(keyword) = keyword else {
        return;
    };
    match search_strategy(keyword) {
        SearchStrategy::FullText => {
            builder
                .push(" AND search_document @@ websearch_to_tsquery('simple',")
                .push_bind(keyword)
                .push(")");
        }
        SearchStrategy::Trigram => {
            builder
                .push(" AND searchable_content ILIKE ")
                .push_bind(like_pattern(keyword))
                .push(" ESCAPE '\\'");
        }
        SearchStrategy::UnicodeNgram => {
            builder.push(" AND search_ngrams @> system_log_search_ngrams(").push_bind(keyword).push(")");
        }
    }
}

fn push_levels(builder: &mut QueryBuilder<Postgres>, filter: &SystemLogFilter) {
    if filter.levels.is_empty() {
        return;
    }
    let mut values = builder.push(" AND level IN (").separated(",");
    for level in &filter.levels {
        values.push_bind(level.code());
    }
    values.push_unseparated(")");
}

fn push_time_range(builder: &mut QueryBuilder<Postgres>, begin: Option<OffsetDateTime>, end: Option<OffsetDateTime>) {
    if let Some(begin) = begin {
        builder.push(" AND occurred_at>=").push_bind(begin);
    }
    if let Some(end) = end {
        builder.push(" AND occurred_at<=").push_bind(end);
    }
}

fn empty_slice() -> SystemLogCursorSlice {
    SystemLogCursorSlice {
        items: Vec::new(),
        snapshot: None,
        has_next: false,
        has_previous: false,
    }
}

fn empty_export_slice() -> SystemLogExportSlice {
    SystemLogExportSlice {
        items: Vec::new(),
        snapshot: None,
        has_next: false,
    }
}

pub(super) fn like_pattern(value: &str) -> String {
    format!("%{}%", escape_like(value))
}

fn target_prefix_pattern(value: &str) -> String {
    format!("{}::%", escape_like(value))
}

fn escape_like(value: &str) -> String {
    value.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

fn search_strategy(keyword: &str) -> SearchStrategy {
    if keyword.chars().count() <= 2 {
        return SearchStrategy::UnicodeNgram;
    }
    if full_text_keyword(keyword) {
        return SearchStrategy::FullText;
    }
    SearchStrategy::Trigram
}

fn full_text_keyword(keyword: &str) -> bool {
    let words = keyword.split_ascii_whitespace().collect::<Vec<_>>();
    words.len() > 1 && words.iter().all(|word| word.bytes().all(|value| value.is_ascii_alphanumeric()))
}

#[cfg(test)]
mod tests {
    use super::{SearchStrategy, like_pattern, search_strategy, target_prefix_pattern};

    #[test]
    fn keyword_pattern_escapes_sql_wildcards() {
        assert_eq!(like_pattern(r"a%b_c\d"), r"%a\%b\_c\\d%");
    }

    #[test]
    fn search_strategy_keeps_short_terms_on_the_unicode_ngram_index() {
        assert_eq!(search_strategy("中"), SearchStrategy::UnicodeNgram);
        assert_eq!(search_strategy("中文"), SearchStrategy::UnicodeNgram);
        assert_eq!(search_strategy("abc"), SearchStrategy::Trigram);
    }

    #[test]
    fn search_strategy_uses_full_text_for_multi_word_ascii_searches() {
        assert_eq!(search_strategy("request failed"), SearchStrategy::FullText);
        assert_eq!(search_strategy("请求失败"), SearchStrategy::Trigram);
    }

    #[test]
    fn target_prefix_pattern_keeps_module_boundaries_and_escapes_wildcards() {
        assert_eq!(target_prefix_pattern("user"), "user::%");
        assert_eq!(target_prefix_pattern("user_%"), r"user\_\%::%");
    }
}
