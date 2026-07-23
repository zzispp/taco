use std::collections::HashSet;

use crate::BusinessType;

pub const API_PREFIX: &str = "/api";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EndpointMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl EndpointMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }

    const fn supports_operation_audit(self) -> bool {
        !matches!(self, Self::Get)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointPermissionRequirement {
    AllOf(&'static [&'static str]),
    AnyOf(&'static [&'static str]),
}

impl EndpointPermissionRequirement {
    pub const fn all_of(values: &'static [&'static str]) -> Self {
        Self::AllOf(values)
    }

    pub const fn any_of(values: &'static [&'static str]) -> Self {
        Self::AnyOf(values)
    }

    pub const fn values(self) -> &'static [&'static str] {
        match self {
            Self::AllOf(values) | Self::AnyOf(values) => values,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndpointPermission {
    pub handler: &'static str,
    pub requirement: EndpointPermissionRequirement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointAccess {
    Public,
    SelfAuthenticated,
    Authenticated,
    Permission(EndpointPermission),
    DataScopedPermission(EndpointPermission),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestCapture {
    Sanitized,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationEndpointAudit {
    pub title_key: &'static str,
    pub business_type: BusinessType,
    pub handler: &'static str,
    pub request_capture: RequestCapture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointAudit {
    /// A conventional GET endpoint with no audit event.
    ReadOnly,
    /// A non-GET endpoint that is intentionally non-mutating, such as a
    /// template download, preview, or captcha exchange.
    ExplicitReadOnly,
    /// An authenticated GET that represents a business download and must be
    /// written to the operation-audit outbox.
    Download(OperationEndpointAudit),
    Operation(OperationEndpointAudit),
    Security,
}

impl EndpointAudit {
    pub const fn read_only_for(method: EndpointMethod) -> Self {
        match method {
            EndpointMethod::Get => Self::ReadOnly,
            EndpointMethod::Post | EndpointMethod::Put | EndpointMethod::Delete => Self::ExplicitReadOnly,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndpointSpec {
    pub method: EndpointMethod,
    pub path: &'static str,
    pub access: EndpointAccess,
    pub audit: EndpointAudit,
}

impl EndpointSpec {
    pub fn api_route_path(self) -> &'static str {
        self.path
            .strip_prefix(API_PREFIX)
            .filter(|path| path.starts_with('/'))
            .expect("endpoint specs must use an absolute /api path")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndpointManifest {
    segments: &'static [&'static [EndpointSpec]],
}

impl EndpointManifest {
    pub const fn new(segments: &'static [&'static [EndpointSpec]]) -> Self {
        Self { segments }
    }

    pub fn iter(self) -> impl Iterator<Item = &'static EndpointSpec> {
        self.segments.iter().copied().flat_map(|segment| segment.iter())
    }

    pub fn validate(self) -> Result<(), EndpointSpecError> {
        validate_endpoint_iter(self.iter().copied())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EndpointSpecError {
    #[error("endpoint path must be an absolute /api path: {path}")]
    InvalidPath { path: &'static str },
    #[error("endpoint permission handler must not be blank: {path}")]
    MissingPermissionHandler { path: &'static str },
    #[error("endpoint permission requirement must not be empty: {path}")]
    EmptyPermissionRequirement { path: &'static str },
    #[error("operation audit cannot use GET: {path}")]
    OperationOnRead { path: &'static str },
    #[error("download audit must use GET: {path}")]
    DownloadOnWrite { path: &'static str },
    #[error("non-GET endpoint must declare explicit read-only, operation, or security audit policy: {method} {path}")]
    NonGetReadOnly { method: &'static str, path: &'static str },
    #[error("explicit read-only audit policy is only valid for non-GET endpoints: {path}")]
    ExplicitReadOnlyOnGet { path: &'static str },
    #[error("operation audit handler must not be blank: {path}")]
    MissingOperationHandler { path: &'static str },
    #[error("operation audit title key must not be blank: {path}")]
    MissingOperationTitle { path: &'static str },
    #[error("duplicate endpoint specification: {method} {path}")]
    Duplicate { method: &'static str, path: &'static str },
}

pub fn validate_endpoint_specs(specs: &[EndpointSpec]) -> Result<(), EndpointSpecError> {
    validate_endpoint_iter(specs.iter().copied())
}

fn validate_endpoint_iter(specs: impl IntoIterator<Item = EndpointSpec>) -> Result<(), EndpointSpecError> {
    let mut declared = HashSet::new();
    for spec in specs {
        validate_endpoint(spec)?;
        if !declared.insert((spec.method, spec.path)) {
            return Err(EndpointSpecError::Duplicate {
                method: spec.method.as_str(),
                path: spec.path,
            });
        }
    }
    Ok(())
}

fn validate_endpoint(spec: EndpointSpec) -> Result<(), EndpointSpecError> {
    if !valid_api_path(spec.path) {
        return Err(EndpointSpecError::InvalidPath { path: spec.path });
    }
    validate_access(spec)?;
    validate_audit(spec)
}

fn valid_api_path(path: &str) -> bool {
    path.strip_prefix(API_PREFIX).is_some_and(|nested| nested.starts_with('/') && nested.len() > 1)
}

fn validate_access(spec: EndpointSpec) -> Result<(), EndpointSpecError> {
    let permission = match spec.access {
        EndpointAccess::Permission(permission) | EndpointAccess::DataScopedPermission(permission) => permission,
        EndpointAccess::Public | EndpointAccess::SelfAuthenticated | EndpointAccess::Authenticated => return Ok(()),
    };
    if permission.handler.trim().is_empty() {
        return Err(EndpointSpecError::MissingPermissionHandler { path: spec.path });
    }
    if permission.requirement.values().is_empty() {
        return Err(EndpointSpecError::EmptyPermissionRequirement { path: spec.path });
    }
    Ok(())
}

fn validate_audit(spec: EndpointSpec) -> Result<(), EndpointSpecError> {
    match spec.audit {
        EndpointAudit::ReadOnly if !matches!(spec.method, EndpointMethod::Get) => {
            return Err(EndpointSpecError::NonGetReadOnly {
                method: spec.method.as_str(),
                path: spec.path,
            });
        }
        EndpointAudit::ExplicitReadOnly if matches!(spec.method, EndpointMethod::Get) => {
            return Err(EndpointSpecError::ExplicitReadOnlyOnGet { path: spec.path });
        }
        EndpointAudit::Operation(operation) => {
            if !spec.method.supports_operation_audit() {
                return Err(EndpointSpecError::OperationOnRead { path: spec.path });
            }
            validate_operation(operation, spec.path)?;
        }
        EndpointAudit::Download(operation) => {
            if !matches!(spec.method, EndpointMethod::Get) {
                return Err(EndpointSpecError::DownloadOnWrite { path: spec.path });
            }
            validate_operation(operation, spec.path)?;
        }
        EndpointAudit::ReadOnly | EndpointAudit::ExplicitReadOnly | EndpointAudit::Security => {}
    }
    Ok(())
}

fn validate_operation(operation: OperationEndpointAudit, path: &'static str) -> Result<(), EndpointSpecError> {
    if operation.handler.trim().is_empty() {
        return Err(EndpointSpecError::MissingOperationHandler { path });
    }
    if operation.title_key.trim().is_empty() {
        return Err(EndpointSpecError::MissingOperationTitle { path });
    }
    Ok(())
}

#[cfg(test)]
#[path = "endpoint_tests.rs"]
mod tests;
