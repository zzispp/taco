use std::path::Path;

use sysinfo::Disks;

use crate::domain::{ByteSize, ProviderCapacity};
use crate::{FileError, FileResult};

use super::support::provider_io;

pub(super) fn disk_capacity(root: &Path) -> FileResult<ProviderCapacity> {
    let root = std::fs::canonicalize(root).map_err(|_| provider_io("canonicalize local provider root"))?;
    let disks = Disks::new_with_refreshed_list();
    let disk = disks
        .iter()
        .filter(|disk| root.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().components().count())
        .ok_or(FileError::ProviderUnavailable {
            operation: "locate backing disk",
        })?;
    ProviderCapacity::bounded(ByteSize::from_bytes(disk.total_space()), ByteSize::from_bytes(disk.available_space()))
}
