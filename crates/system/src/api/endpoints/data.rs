use audit_contract::{BusinessType, EndpointAccess, EndpointMethod, EndpointPermissionRequirement, EndpointSpec};

use super::{operation, operation_without_request, permission, read};

const DICT_TYPES: &str = "/api/system/dict-types";
const DICT_DATA: &str = "/api/system/dict-data";
const CONFIGS: &str = "/api/system/configs";

pub(in crate::api) const DICT_TYPES_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    DICT_TYPES,
    permission("list_dict_types", EndpointPermissionRequirement::all_of(&["system:dict:list"])),
);
pub(in crate::api) const DICT_TYPES_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: DICT_TYPES,
    access: permission("create_dict_type", EndpointPermissionRequirement::all_of(&["system:dict:add"])),
    audit: operation("audit.module.dict_type", BusinessType::Insert, "system::create_dict_type"),
};
pub(in crate::api) const DICT_TYPES_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/dict-types/export",
    access: permission("export_dict_types", EndpointPermissionRequirement::all_of(&["system:dict:export"])),
    audit: operation("audit.module.dict_type", BusinessType::Export, "system::export_dict_types"),
};
pub(in crate::api) const DICT_TYPES_OPTIONS: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/dict-types/options",
    permission("dict_type_options", EndpointPermissionRequirement::all_of(&["system:dict:list"])),
);
pub(in crate::api) const DICT_TYPES_CACHE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/dict-types/cache",
    access: permission("refresh_dict_cache", EndpointPermissionRequirement::all_of(&["system:dict:remove"])),
    audit: operation("audit.module.dict_type", BusinessType::Clean, "system::refresh_dict_cache"),
};
pub(in crate::api) const DICT_TYPES_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/dict-types/batch",
    access: permission("delete_dict_types", EndpointPermissionRequirement::all_of(&["system:dict:remove"])),
    audit: operation("audit.module.dict_type", BusinessType::Delete, "system::delete_dict_types"),
};
pub(in crate::api) const DICT_TYPE_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/dict-types/{id}",
    permission("get_dict_type", EndpointPermissionRequirement::all_of(&["system:dict:query"])),
);
pub(in crate::api) const DICT_TYPE_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/dict-types/{id}",
    access: permission("replace_dict_type", EndpointPermissionRequirement::all_of(&["system:dict:edit"])),
    audit: operation("audit.module.dict_type", BusinessType::Update, "system::replace_dict_type"),
};
pub(in crate::api) const DICT_TYPE_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/dict-types/{id}",
    access: permission("delete_dict_type", EndpointPermissionRequirement::all_of(&["system:dict:remove"])),
    audit: operation("audit.module.dict_type", BusinessType::Delete, "system::delete_dict_type"),
};

pub(in crate::api) const DICT_DATA_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    DICT_DATA,
    permission("list_dict_data", EndpointPermissionRequirement::all_of(&["system:dict:list"])),
);
pub(in crate::api) const DICT_DATA_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: DICT_DATA,
    access: permission("create_dict_data", EndpointPermissionRequirement::all_of(&["system:dict:add"])),
    audit: operation("audit.module.dict_data", BusinessType::Insert, "system::create_dict_data"),
};
pub(in crate::api) const DICT_DATA_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/dict-data/export",
    access: permission("export_dict_data", EndpointPermissionRequirement::all_of(&["system:dict:export"])),
    audit: operation("audit.module.dict_data", BusinessType::Export, "system::export_dict_data"),
};
pub(in crate::api) const DICT_DATA_BY_TYPE: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/dict-data/type/{dict_type}",
    permission("dict_data_by_type", EndpointPermissionRequirement::all_of(&["system:dict:list"])),
);
pub(in crate::api) const DICT_DATA_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/dict-data/batch",
    access: permission("delete_dict_data_batch", EndpointPermissionRequirement::all_of(&["system:dict:remove"])),
    audit: operation("audit.module.dict_data", BusinessType::Delete, "system::delete_dict_data_batch"),
};
pub(in crate::api) const DICT_DATA_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/dict-data/{id}",
    permission("get_dict_data", EndpointPermissionRequirement::all_of(&["system:dict:query"])),
);
pub(in crate::api) const DICT_DATA_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/dict-data/{id}",
    access: permission("replace_dict_data", EndpointPermissionRequirement::all_of(&["system:dict:edit"])),
    audit: operation("audit.module.dict_data", BusinessType::Update, "system::replace_dict_data"),
};
pub(in crate::api) const DICT_DATA_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/dict-data/{id}",
    access: permission("delete_dict_data", EndpointPermissionRequirement::all_of(&["system:dict:remove"])),
    audit: operation("audit.module.dict_data", BusinessType::Delete, "system::delete_dict_data"),
};

pub(in crate::api) const CONFIGS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    CONFIGS,
    permission("list_configs", EndpointPermissionRequirement::all_of(&["system:config:list"])),
);
pub(in crate::api) const CONFIGS_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: CONFIGS,
    access: permission("create_config", EndpointPermissionRequirement::all_of(&["system:config:add"])),
    audit: operation_without_request("audit.module.config", BusinessType::Insert, "system::create_config"),
};
pub(in crate::api) const CONFIGS_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/configs/export",
    access: permission("export_configs", EndpointPermissionRequirement::all_of(&["system:config:export"])),
    audit: operation_without_request("audit.module.config", BusinessType::Export, "system::export_configs"),
};
pub(in crate::api) const CONFIGS_CACHE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/configs/cache",
    access: permission("refresh_config_cache", EndpointPermissionRequirement::all_of(&["system:config:remove"])),
    audit: operation_without_request("audit.module.config", BusinessType::Clean, "system::refresh_config_cache"),
};
pub(in crate::api) const CONFIGS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/configs/batch",
    access: permission("delete_configs", EndpointPermissionRequirement::all_of(&["system:config:remove"])),
    audit: operation_without_request("audit.module.config", BusinessType::Delete, "system::delete_configs"),
};
pub(in crate::api) const CONFIG_BY_KEY: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/configs/key/{key}",
    permission("config_by_key", EndpointPermissionRequirement::all_of(&["system:config:query"])),
);
pub(in crate::api) const CONFIG_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/configs/{id}",
    permission("get_config", EndpointPermissionRequirement::all_of(&["system:config:query"])),
);
pub(in crate::api) const CONFIG_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/configs/{id}",
    access: permission("replace_config", EndpointPermissionRequirement::all_of(&["system:config:edit"])),
    audit: operation_without_request("audit.module.config", BusinessType::Update, "system::replace_config"),
};
pub(in crate::api) const CONFIG_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/configs/{id}",
    access: permission("delete_config", EndpointPermissionRequirement::all_of(&["system:config:remove"])),
    audit: operation_without_request("audit.module.config", BusinessType::Delete, "system::delete_config"),
};
pub(in crate::api) const PUBLIC_CONFIGS: EndpointSpec = read(EndpointMethod::Get, "/api/app/configs", EndpointAccess::Public);

pub(super) const ENDPOINTS: &[EndpointSpec] = &[
    DICT_TYPES_LIST,
    DICT_TYPES_CREATE,
    DICT_TYPES_EXPORT,
    DICT_TYPES_OPTIONS,
    DICT_TYPES_CACHE,
    DICT_TYPES_DELETE_BATCH,
    DICT_TYPE_GET,
    DICT_TYPE_REPLACE,
    DICT_TYPE_DELETE,
    DICT_DATA_LIST,
    DICT_DATA_CREATE,
    DICT_DATA_EXPORT,
    DICT_DATA_BY_TYPE,
    DICT_DATA_DELETE_BATCH,
    DICT_DATA_GET,
    DICT_DATA_REPLACE,
    DICT_DATA_DELETE,
    CONFIGS_LIST,
    CONFIGS_CREATE,
    CONFIGS_EXPORT,
    CONFIGS_CACHE,
    CONFIGS_DELETE_BATCH,
    CONFIG_BY_KEY,
    CONFIG_GET,
    CONFIG_REPLACE,
    CONFIG_DELETE,
    PUBLIC_CONFIGS,
];
