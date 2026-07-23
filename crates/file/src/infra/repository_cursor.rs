use sqlx::{Postgres, QueryBuilder};

use crate::application::{FileListQuery, FileSpaceQuery};
use crate::domain::ByteSize;
use crate::error::keys;
use crate::{FileError, FileResult};

use super::{EntryRecord, PageCursor, SpaceRecord, format_time, parse_time};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn parse(value: Option<&str>) -> FileResult<Self> {
        match value.unwrap_or("desc") {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(FileError::InvalidInput(keys::SORT_ORDER_INVALID)),
        }
    }

    const fn operator(self) -> &'static str {
        match self {
            Self::Asc => ">",
            Self::Desc => "<",
        }
    }

    const fn sql(self) -> &'static str {
        match self {
            Self::Asc => " ASC",
            Self::Desc => " DESC",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EntrySortField {
    UpdatedAt,
    CreatedAt,
    Name,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::infra) struct EntrySortSpec {
    field: EntrySortField,
    direction: SortDirection,
}

impl EntrySortSpec {
    pub(in crate::infra) fn from_filter(filter: &FileListQuery) -> FileResult<Self> {
        let field = match filter.sort_by.as_deref().unwrap_or("updated_at") {
            "updated_at" => EntrySortField::UpdatedAt,
            "created_at" => EntrySortField::CreatedAt,
            "name" => EntrySortField::Name,
            _ => return Err(FileError::InvalidInput(keys::SORT_FIELD_INVALID)),
        };
        Ok(Self {
            field,
            direction: SortDirection::parse(filter.sort_order.as_deref())?,
        })
    }

    pub(in crate::infra) fn cursor_value(self, record: &EntryRecord) -> String {
        match self.field {
            EntrySortField::UpdatedAt => format_time(record.updated_at),
            EntrySortField::CreatedAt => format_time(record.created_at),
            EntrySortField::Name => record.normalized_name.clone(),
        }
    }

    pub(in crate::infra) fn push_cursor_bound(self, query: &mut QueryBuilder<Postgres>, cursor: Option<&PageCursor>) -> FileResult<()> {
        let Some(cursor) = cursor else { return Ok(()) };
        query
            .push(" AND (")
            .push(self.column())
            .push(",e.entry_id)")
            .push(self.direction.operator())
            .push("(");
        match self.field {
            EntrySortField::UpdatedAt | EntrySortField::CreatedAt => query.push_bind(parse_time(&cursor.sort_value)?),
            EntrySortField::Name => query.push_bind(cursor.sort_value.clone()),
        };
        query.push(",").push_bind(cursor.id.clone()).push(")");
        Ok(())
    }

    pub(in crate::infra) fn push_order(self, query: &mut QueryBuilder<Postgres>) {
        query
            .push(" ORDER BY ")
            .push(self.column())
            .push(self.direction.sql())
            .push(",e.entry_id")
            .push(self.direction.sql());
    }

    fn column(self) -> &'static str {
        match self.field {
            EntrySortField::UpdatedAt => "e.updated_at",
            EntrySortField::CreatedAt => "e.created_at",
            EntrySortField::Name => "e.normalized_name",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpaceSortField {
    OwnerName,
    DepartmentName,
    Status,
    LogicalAssetSize,
    ReservedBytes,
    QuotaBytes,
    UpdatedAt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::infra) struct SpaceSortSpec {
    field: SpaceSortField,
    direction: SortDirection,
}

impl SpaceSortSpec {
    pub(in crate::infra) fn from_filter(filter: &FileSpaceQuery) -> FileResult<Self> {
        let field = match filter.sort_by.as_deref().unwrap_or("updated_at") {
            "owner_name" => SpaceSortField::OwnerName,
            "department_name" => SpaceSortField::DepartmentName,
            "status" => SpaceSortField::Status,
            "logical_asset_size" => SpaceSortField::LogicalAssetSize,
            "reserved_bytes" => SpaceSortField::ReservedBytes,
            "quota_bytes" => SpaceSortField::QuotaBytes,
            "updated_at" => SpaceSortField::UpdatedAt,
            _ => return Err(FileError::InvalidInput(keys::SORT_FIELD_INVALID)),
        };
        Ok(Self {
            field,
            direction: SortDirection::parse(filter.sort_order.as_deref())?,
        })
    }

    pub(in crate::infra) fn cursor_value(self, record: &SpaceRecord, default_quota: ByteSize) -> FileResult<String> {
        match self.field {
            SpaceSortField::OwnerName => Ok(record.owner_name.clone()),
            SpaceSortField::DepartmentName => Ok(record.department_name.clone().unwrap_or_default()),
            SpaceSortField::Status => Ok(record.status.clone()),
            SpaceSortField::LogicalAssetSize => Ok(record.active_bytes.to_string()),
            SpaceSortField::ReservedBytes => Ok(record.reserved_bytes.to_string()),
            SpaceSortField::QuotaBytes => quota_value(record, default_quota).map(|value| value.to_string()),
            SpaceSortField::UpdatedAt => Ok(format_time(record.updated_at)),
        }
    }

    pub(in crate::infra) fn push_cursor_bound(
        self,
        query: &mut QueryBuilder<Postgres>,
        cursor: Option<&PageCursor>,
        default_quota: ByteSize,
    ) -> FileResult<()> {
        let Some(cursor) = cursor else { return Ok(()) };
        query.push(" AND (");
        self.push_column(query, default_quota)?;
        query.push(",s.space_id)").push(self.direction.operator()).push("(");
        self.push_cursor_value(query, cursor)?;
        query.push(",").push_bind(cursor.id.clone()).push(")");
        Ok(())
    }

    pub(in crate::infra) fn push_order(self, query: &mut QueryBuilder<Postgres>, default_quota: ByteSize) -> FileResult<()> {
        query.push(" ORDER BY ");
        self.push_column(query, default_quota)?;
        query.push(self.direction.sql()).push(",s.space_id").push(self.direction.sql());
        Ok(())
    }

    fn push_column(self, query: &mut QueryBuilder<Postgres>, default_quota: ByteSize) -> FileResult<()> {
        match self.field {
            SpaceSortField::OwnerName => {
                query.push("s.owner_name");
            }
            SpaceSortField::DepartmentName => {
                query.push("COALESCE(d.dept_name,'')");
            }
            SpaceSortField::Status => {
                query.push("s.status");
            }
            SpaceSortField::LogicalAssetSize => {
                query.push("s.active_bytes");
            }
            SpaceSortField::ReservedBytes => {
                query.push("s.reserved_bytes");
            }
            SpaceSortField::QuotaBytes => {
                query
                    .push("COALESCE(s.quota_override_bytes,")
                    .push_bind(default_quota_value(default_quota)?)
                    .push(")");
            }
            SpaceSortField::UpdatedAt => {
                query.push("s.updated_at");
            }
        }
        Ok(())
    }

    fn push_cursor_value(self, query: &mut QueryBuilder<Postgres>, cursor: &PageCursor) -> FileResult<()> {
        match self.field {
            SpaceSortField::OwnerName | SpaceSortField::DepartmentName | SpaceSortField::Status => {
                query.push_bind(cursor.sort_value.clone());
            }
            SpaceSortField::LogicalAssetSize | SpaceSortField::ReservedBytes | SpaceSortField::QuotaBytes => {
                query.push_bind(parse_cursor_integer(&cursor.sort_value)?);
            }
            SpaceSortField::UpdatedAt => {
                query.push_bind(parse_time(&cursor.sort_value)?);
            }
        };
        Ok(())
    }
}

fn quota_value(record: &SpaceRecord, default_quota: ByteSize) -> FileResult<i64> {
    Ok(record.quota_override_bytes.unwrap_or(default_quota_value(default_quota)?))
}

fn default_quota_value(default_quota: ByteSize) -> FileResult<i64> {
    i64::try_from(default_quota.bytes()).map_err(|_| FileError::InvalidInput(keys::QUOTA_TOO_LARGE))
}

fn parse_cursor_integer(value: &str) -> FileResult<i64> {
    value.parse().map_err(|_| FileError::InvalidInput(keys::CURSOR_MALFORMED))
}

#[cfg(test)]
#[path = "repository_cursor_tests.rs"]
mod tests;
