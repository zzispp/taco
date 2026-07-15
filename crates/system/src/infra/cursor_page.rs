use kernel::pagination::{CursorDirection, CursorPage};

use crate::{
    application::{SystemBoundary, SystemCursorCodec, SystemResult, TimeIdPoint, point},
    domain::{ConfigItem, Dept, DictData, DictType, Post},
};

use super::{
    mapping::{config, dept, dict_data, dict_type, post, storage_error},
    record::{ConfigRecord, DeptRecord, DictDataRecord, DictTypeRecord, PostRecord},
};

pub(super) struct PageNavigation {
    boundary: Option<SystemBoundary>,
    pub(super) direction: CursorDirection,
    pub(super) limit: u64,
    from_cursor: bool,
}

pub(super) trait CursorRecord: Sized {
    type Item;

    fn boundary(&self) -> SystemBoundary;
    fn snapshot(&self) -> SystemResult<TimeIdPoint>;
    fn into_item(self) -> SystemResult<Self::Item>;
}

pub(super) struct PageBuildContext<'a> {
    pub(super) codec: &'a SystemCursorCodec,
    pub(super) snapshot: &'a TimeIdPoint,
    pub(super) navigation: &'a PageNavigation,
}

pub(super) fn navigation(decoded: Option<&crate::application::SystemDecodedCursor>, limit: u64) -> PageNavigation {
    PageNavigation {
        boundary: decoded.map(|cursor| cursor.boundary.clone()),
        direction: decoded.map_or(CursorDirection::Next, |cursor| cursor.direction),
        limit,
        from_cursor: decoded.is_some(),
    }
}

pub(super) fn build_page<R: CursorRecord>(mut records: Vec<R>, context: PageBuildContext<'_>) -> SystemResult<CursorPage<R::Item>> {
    let requested = usize::try_from(context.navigation.limit).map_err(numeric_error)?;
    let has_extra = records.len() > requested;
    records.truncate(requested);
    if context.navigation.direction == CursorDirection::Previous {
        records.reverse();
    }
    let (next, previous) = page_cursors(&records, &context, has_extra)?;
    let items = records.into_iter().map(CursorRecord::into_item).collect::<SystemResult<Vec<_>>>()?;
    Ok(CursorPage::new(items, next, previous))
}

fn page_cursors<R: CursorRecord>(records: &[R], context: &PageBuildContext<'_>, has_extra: bool) -> SystemResult<(Option<String>, Option<String>)> {
    let Some(first) = records.first() else {
        return empty_cursors(context);
    };
    let last = records.last().expect("a non-empty system cursor page has a last record");
    let has_previous = context.navigation.from_cursor && (context.navigation.direction == CursorDirection::Next || has_extra);
    let has_next = has_extra || (context.navigation.from_cursor && context.navigation.direction == CursorDirection::Previous);
    let next = has_next
        .then(|| context.codec.encode(CursorDirection::Next, &last.boundary(), context.snapshot))
        .transpose()?;
    let previous = has_previous
        .then(|| context.codec.encode(CursorDirection::Previous, &first.boundary(), context.snapshot))
        .transpose()?;
    Ok((next, previous))
}

fn empty_cursors(context: &PageBuildContext<'_>) -> SystemResult<(Option<String>, Option<String>)> {
    let Some(boundary) = &context.navigation.boundary else {
        return Ok((None, None));
    };
    match context.navigation.direction {
        CursorDirection::Next => Ok((None, Some(context.codec.encode(CursorDirection::Previous, boundary, context.snapshot)?))),
        CursorDirection::Previous => Ok((Some(context.codec.encode(CursorDirection::Next, boundary, context.snapshot)?), None)),
    }
}

impl CursorRecord for DeptRecord {
    type Item = Dept;

    fn boundary(&self) -> SystemBoundary {
        SystemBoundary::Dept {
            parent_id: self.parent_id.clone(),
            order_num: self.order_num,
            dept_id: self.dept_id.clone(),
        }
    }

    fn snapshot(&self) -> SystemResult<TimeIdPoint> {
        point(self.create_time, self.dept_id.clone())
    }

    fn into_item(self) -> SystemResult<Self::Item> {
        dept(self).map_err(storage_error)
    }
}

macro_rules! simple_record {
    ($record:ty, $item:ty, $boundary:expr, $id:expr, $mapper:expr) => {
        impl CursorRecord for $record {
            type Item = $item;

            fn boundary(&self) -> SystemBoundary {
                $boundary(self)
            }

            fn snapshot(&self) -> SystemResult<TimeIdPoint> {
                point(self.create_time, $id(self))
            }

            fn into_item(self) -> SystemResult<Self::Item> {
                $mapper(self).map_err(storage_error)
            }
        }
    };
}

simple_record!(
    PostRecord,
    Post,
    |record: &PostRecord| SystemBoundary::Post {
        post_sort: record.post_sort,
        post_id: record.post_id.clone()
    },
    |record: &PostRecord| record.post_id.clone(),
    post
);
simple_record!(
    DictTypeRecord,
    DictType,
    |record: &DictTypeRecord| SystemBoundary::DictType {
        dict_id: record.dict_id.clone()
    },
    |record: &DictTypeRecord| record.dict_id.clone(),
    dict_type
);
simple_record!(
    DictDataRecord,
    DictData,
    |record: &DictDataRecord| SystemBoundary::DictData {
        dict_sort: record.dict_sort,
        dict_code: record.dict_code.clone()
    },
    |record: &DictDataRecord| record.dict_code.clone(),
    dict_data
);
simple_record!(
    ConfigRecord,
    ConfigItem,
    |record: &ConfigRecord| SystemBoundary::Config {
        config_id: record.config_id.clone()
    },
    |record: &ConfigRecord| record.config_id.clone(),
    config
);

fn numeric_error(error: impl std::fmt::Display) -> crate::application::SystemError {
    crate::application::SystemError::Infrastructure(format!("system cursor numeric conversion failed: {error}"))
}
