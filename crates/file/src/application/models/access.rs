use serde::{Deserialize, Serialize};

use crate::domain::{DirectoryId, SpaceId};

/// The caller's effective space visibility, independent from RBAC transport types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileAccessScope {
    pub user_id: String,
    pub mode: FileScopeMode,
    pub department_id: Option<String>,
    pub department_ids: Vec<String>,
    pub can_manage_uploads: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileScopeMode {
    All,
    SelfOnly,
    Department,
    DepartmentAndChildren,
    Custom,
}

impl FileAccessScope {
    pub fn self_only(user_id: impl Into<String>, department_id: Option<String>) -> Self {
        Self {
            user_id: user_id.into(),
            mode: FileScopeMode::SelfOnly,
            department_id,
            department_ids: Vec::new(),
            can_manage_uploads: false,
        }
    }

    pub fn all(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            mode: FileScopeMode::All,
            department_id: None,
            department_ids: Vec::new(),
            can_manage_uploads: false,
        }
    }

    pub fn scoped(user_id: impl Into<String>, mode: FileScopeMode, department_id: Option<String>, department_ids: Vec<String>) -> Self {
        Self {
            user_id: user_id.into(),
            mode,
            department_id,
            department_ids,
            can_manage_uploads: false,
        }
    }

    pub fn with_upload_management(mut self, allowed: bool) -> Self {
        self.can_manage_uploads = allowed;
        self
    }

    pub fn allows_owner(&self, owner_user_id: &str, owner_dept_id: Option<&str>) -> bool {
        if self.mode == FileScopeMode::All || owner_user_id == self.user_id {
            return true;
        }
        match self.mode {
            FileScopeMode::Department | FileScopeMode::DepartmentAndChildren => self.department_id.as_deref() == owner_dept_id,
            FileScopeMode::Custom => self.department_ids.iter().any(|id| Some(id.as_str()) == owner_dept_id),
            FileScopeMode::All | FileScopeMode::SelfOnly => false,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileListQuery {
    pub cursor: Option<String>,
    pub space_id: Option<SpaceId>,
    pub parent_id: Option<DirectoryId>,
    pub kind: Option<String>,
    pub search: Option<String>,
    pub extension: Option<String>,
    pub mime_type: Option<String>,
    pub tag: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub trashed: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileSpaceQuery {
    pub cursor: Option<String>,
    pub owner_user_id: Option<String>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}
