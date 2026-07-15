use kernel::pagination::{CursorDirection, CursorPage, DecodedCursor};
use types::rbac::{Menu, Role, RoleUser};

use crate::{
    application::{
        RbacResult,
        cursor::{MenuBoundary, MenuCursorCodec, RoleBoundary, RoleCursorCodec, RoleUserCursorCodec, TimeIdPoint, point},
    },
    infra::{
        mapping::{menu, role, role_user, storage_error},
        records::{MenuRecord, RoleRecord, RoleUserRecord},
    },
};

pub(super) struct PageNavigation<B> {
    boundary: Option<B>,
    pub(super) direction: CursorDirection,
    pub(super) limit: u64,
    from_cursor: bool,
}

pub(super) trait CursorRecord<B>: Sized {
    type Item;

    fn boundary(&self) -> RbacResult<B>;
    fn into_item(self) -> RbacResult<Self::Item>;
}

pub(super) trait CursorEncoder<B, S> {
    fn encode_page_cursor(&self, direction: CursorDirection, boundary: &B, snapshot: &S) -> RbacResult<String>;
}

pub(super) struct PageBuildContext<'a, B, S, C> {
    pub(super) codec: &'a C,
    pub(super) snapshot: &'a S,
    pub(super) navigation: &'a PageNavigation<B>,
}

pub(super) fn navigation<B: Clone, S>(decoded: Option<&DecodedCursor<B, S>>, limit: u64) -> PageNavigation<B> {
    PageNavigation {
        boundary: decoded.map(|cursor| cursor.boundary.clone()),
        direction: decoded.map_or(CursorDirection::Next, |cursor| cursor.direction),
        limit,
        from_cursor: decoded.is_some(),
    }
}

pub(super) fn build_page<R, B, S, C>(mut records: Vec<R>, context: PageBuildContext<'_, B, S, C>) -> RbacResult<CursorPage<R::Item>>
where
    R: CursorRecord<B>,
    C: CursorEncoder<B, S>,
{
    let requested = usize::try_from(context.navigation.limit).map_err(numeric_error)?;
    let has_extra = records.len() > requested;
    records.truncate(requested);
    if context.navigation.direction == CursorDirection::Previous {
        records.reverse();
    }
    let (next, previous) = page_cursors(&records, &context, has_extra)?;
    let items = records.into_iter().map(CursorRecord::into_item).collect::<RbacResult<Vec<_>>>()?;
    Ok(CursorPage::new(items, next, previous))
}

fn page_cursors<R, B, S, C>(records: &[R], context: &PageBuildContext<'_, B, S, C>, has_extra: bool) -> RbacResult<(Option<String>, Option<String>)>
where
    R: CursorRecord<B>,
    C: CursorEncoder<B, S>,
{
    let Some(first) = records.first() else {
        return empty_page_cursors(context);
    };
    let last = records.last().expect("a non-empty cursor page has a last record");
    let has_previous = context.navigation.from_cursor && (context.navigation.direction == CursorDirection::Next || has_extra);
    let has_next = has_extra || (context.navigation.from_cursor && context.navigation.direction == CursorDirection::Previous);
    let next = has_next
        .then(|| context.codec.encode_page_cursor(CursorDirection::Next, &last.boundary()?, context.snapshot))
        .transpose()?;
    let previous = has_previous
        .then(|| {
            context
                .codec
                .encode_page_cursor(CursorDirection::Previous, &first.boundary()?, context.snapshot)
        })
        .transpose()?;
    Ok((next, previous))
}

fn empty_page_cursors<B, S, C>(context: &PageBuildContext<'_, B, S, C>) -> RbacResult<(Option<String>, Option<String>)>
where
    C: CursorEncoder<B, S>,
{
    let Some(boundary) = &context.navigation.boundary else {
        return Ok((None, None));
    };
    match context.navigation.direction {
        CursorDirection::Next => Ok((
            None,
            Some(context.codec.encode_page_cursor(CursorDirection::Previous, boundary, context.snapshot)?),
        )),
        CursorDirection::Previous => Ok((Some(context.codec.encode_page_cursor(CursorDirection::Next, boundary, context.snapshot)?), None)),
    }
}

impl CursorEncoder<RoleBoundary, TimeIdPoint> for RoleCursorCodec {
    fn encode_page_cursor(&self, direction: CursorDirection, boundary: &RoleBoundary, snapshot: &TimeIdPoint) -> RbacResult<String> {
        self.encode(direction, boundary, snapshot)
    }
}

impl CursorEncoder<MenuBoundary, TimeIdPoint> for MenuCursorCodec {
    fn encode_page_cursor(&self, direction: CursorDirection, boundary: &MenuBoundary, snapshot: &TimeIdPoint) -> RbacResult<String> {
        self.encode(direction, boundary, snapshot)
    }
}

impl CursorEncoder<TimeIdPoint, TimeIdPoint> for RoleUserCursorCodec {
    fn encode_page_cursor(&self, direction: CursorDirection, boundary: &TimeIdPoint, snapshot: &TimeIdPoint) -> RbacResult<String> {
        self.encode(direction, boundary, snapshot)
    }
}

impl CursorRecord<RoleBoundary> for RoleRecord {
    type Item = Role;

    fn boundary(&self) -> RbacResult<RoleBoundary> {
        Ok(RoleBoundary {
            role_sort: self.role_sort,
            role_id: self.role_id.clone(),
        })
    }

    fn into_item(self) -> RbacResult<Self::Item> {
        role(self).map_err(storage_error)
    }
}

impl CursorRecord<MenuBoundary> for MenuRecord {
    type Item = Menu;

    fn boundary(&self) -> RbacResult<MenuBoundary> {
        Ok(MenuBoundary {
            parent_id: self.parent_id.clone(),
            order_num: self.order_num,
            menu_id: self.menu_id.clone(),
        })
    }

    fn into_item(self) -> RbacResult<Self::Item> {
        Ok(menu(self))
    }
}

impl CursorRecord<TimeIdPoint> for RoleUserRecord {
    type Item = RoleUser;

    fn boundary(&self) -> RbacResult<TimeIdPoint> {
        point(self.create_time, self.user_id.clone())
    }

    fn into_item(self) -> RbacResult<Self::Item> {
        Ok(role_user(self))
    }
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::RbacError {
    crate::application::RbacError::Infrastructure(format!("RBAC cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
#[path = "cursor_page_tests.rs"]
mod tests;
