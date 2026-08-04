use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::FileResult;
use crate::application::ObjectKey;
use crate::domain::{PartNumber, UploadId};

use super::support::provider_io;

const FILES_DIRECTORY: &str = "files";
const OBJECTS_DIRECTORY: &str = "objects";
const PARTS_DIRECTORY: &str = "parts";
const DERIVATIVES_DIRECTORY: &str = "derivatives";
const MANIFEST_FILE: &str = "manifest.json";
const OBJECT_DATA_SUFFIX: &str = "data";
const OBJECT_METADATA_SUFFIX: &str = "json";

#[derive(Debug)]
pub(super) struct LocalPaths {
    pub(super) root: PathBuf,
    objects: PathBuf,
    parts: PathBuf,
    derivatives: PathBuf,
}

impl LocalPaths {
    pub(super) fn new(data_directory: &Path) -> FileResult<Self> {
        let root = data_directory.join(FILES_DIRECTORY);
        let paths = Self {
            objects: root.join(OBJECTS_DIRECTORY),
            parts: root.join(PARTS_DIRECTORY),
            derivatives: root.join(DERIVATIVES_DIRECTORY),
            root,
        };
        for path in [&paths.root, &paths.objects, &paths.parts, &paths.derivatives] {
            std::fs::create_dir_all(path).map_err(|_| provider_io("create local provider directory"))?;
        }
        Ok(paths)
    }

    pub(super) fn session(&self, id: UploadId) -> PathBuf {
        self.parts.join(id.to_string())
    }

    pub(super) fn manifest(&self, id: UploadId) -> PathBuf {
        self.session(id).join(MANIFEST_FILE)
    }

    pub(super) fn part(&self, id: UploadId, number: PartNumber) -> PathBuf {
        self.session(id).join(format!("{}.part", number.value()))
    }

    pub(super) fn incoming_part(&self, id: UploadId) -> PathBuf {
        self.session(id).join(format!(".incoming-{}", Uuid::now_v7()))
    }

    pub(super) fn completing(&self, id: UploadId) -> PathBuf {
        self.session(id).join(format!(".completing-{}", Uuid::now_v7()))
    }

    pub(super) fn object_data(&self, key: &ObjectKey) -> PathBuf {
        self.objects.join(key.as_str()).with_extension(OBJECT_DATA_SUFFIX)
    }

    pub(super) fn object_metadata(&self, key: &ObjectKey) -> PathBuf {
        self.objects.join(key.as_str()).with_extension(OBJECT_METADATA_SUFFIX)
    }
}
