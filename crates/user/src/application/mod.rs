mod audited;
mod avatar;
mod avatar_normalization;
mod bootstrap;
mod config;
pub(crate) mod cursor;
mod error;
mod ports;
mod service;

pub use audited::{AuditedPasswordChange, AuditedUserRepository, UserImportWrite};
pub use avatar::{AvatarFile, AvatarOwner, AvatarProjection, AvatarProjectionBody, AvatarProjectionStorage, AvatarStorage, NormalizedAvatar};
pub use avatar_normalization::normalize_avatar;
pub use bootstrap::{BootstrapAdministratorInput, BootstrapAdministratorOutcome, BootstrapAdministratorRecord};
pub use config::{
    AvatarConfig, LoginLockConfig, PasswordPolicy, parse_avatar_config, parse_export_batch_config, parse_login_lock_config, parse_password_policy,
};
pub use error::{AppError, AppResult};
pub use ports::{
    AccountVerifier, AuthorizationUser, AvatarConfigProvider, BootstrapAdministratorRepository, LoginFailureStore, LoginLockConfigProvider, OnlineSession,
    OnlineSessionCleanup, OnlineSessionFilter, OnlineSessionPageRequest, OnlineSessionSearch, OnlineSessionStore, PasswordHasher, PasswordPolicyProvider,
    ReplaceUserRecord, SystemConfigProvider, UserAuthRecord, UserExportRequest, UserExportSink, UserImportInput, UserImportMessage, UserImportReport,
    UserImportRow, UserListFilter, UserRepository, UserUseCase, VerifiedLogin,
};
pub use service::UserService;
