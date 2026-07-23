# File Management Design

## Product Model

Every user owns one Personal Asset Space. Capability permissions determine which operations a caller may perform, while the caller's RBAC Data Scope determines which users' spaces those operations may target. The first release has no global shared directory and no user-to-user sharing.

## Navigation

The `文件管理` dashboard menu has three child pages:

- `文件概览` at `/dashboard/file` shows usage, quota, recycle-bin and upload reservations, type distribution, recently created or modified assets, and folders. Users with broader Data Scope can switch between their own space and the managed scope. The first release does not show a mock activity chart or track recently opened files.
- `资产管理` at `/dashboard/file-manager` provides folder navigation, list/grid modes, upload, preview, download, Tags, recycle-bin operations, move, rename, and deletion. Scoped managers receive user and department space selectors.
- `空间管理` at `/dashboard/file-spaces` lists Data Scope-visible spaces and their owners, departments, status, usage, reservations, and quota. Per-space quota changes are managed here; Provider capacity is visible only with its dedicated permission.

The recycle bin is a view inside Asset Management. Info, File Properties, and Tags use the details drawer. Avatar asset selection remains inside the profile avatar dialog. Scheduled cleanup remains in the existing Scheduler administration UI.

Search filters metadata only: name, extension and MIME type, Tags, date, owner space, and recycle-bin state. Content extraction and full-text indexing are not part of the first release.

Batch metadata operations validate every selected asset, permission, scope, conflict, and business reference before committing. Any validation failure rolls the metadata operation back as a whole. Provider-level Permanent Deletion remains an explicitly reported per-object cleanup process.

## Authorization

All three pages and their APIs require capability permissions. Data Scope is applied independently to every target Personal Asset Space: `SelfOnly`, `Department`, `DepartmentAndChildren`, `Custom`, and `All` retain their existing RBAC meanings. No identity marker bypasses capability permissions or Data Scope; administrator access derives from explicit RBAC role and menu bindings.

The capability set is split into `file:asset:list`, `file:asset:query`, `file:asset:download`, `file:asset:upload`, `file:folder:add`, `file:asset:edit`, `file:asset:remove`, `file:asset:restore`, `file:asset:purge`, `file:space:list`, `file:space:quota`, `file:provider:query`, and `file:upload:manage`.

## Content Safety

Personal Asset Spaces accept arbitrary non-empty files within the configured size limit. Only the approved preview formats render inline; every other file is a Download-Only Asset. The first release does not integrate malware scanning and does not expose a Scanner interface with a fake success implementation.

Grid and list views generate bounded thumbnails only for PNG, JPEG, WebP, and the first GIF frame. Other formats use type icons until the user opens Inline Preview or download. Thumbnails are internal derivatives of Stored Objects: they are not Managed Files and do not consume logical Space Quota, but their bytes count toward Managed Physical Usage.

Raster images above 40 million decoded pixels remain storable and downloadable but are not decoded for thumbnails or Inline Preview.

Image preview sources are limited to 32 MiB before temporary staging, and no more than two image validation or thumbnail jobs run concurrently in one process. The limit affects rendering work only; it does not lower the Managed File upload limit.

Text previews read at most 1 MiB, escape rendered content, and mark larger previews as truncated. Complete text remains available through authenticated download.

## Local Provider

The Local Provider derives its root from `<data_directory>/files` and owns `objects/`, `parts/`, and `derivatives/` beneath that root. `data_directory` is a strict startup YAML value loaded through `--config`; relative values are resolved from the YAML file's directory before the Provider receives its absolute runtime path. It is not a separate File Management runtime parameter and YAML changes require a Taco restart. The path is never exposed through a static-file service; all content reads use the authenticated File Management API except Avatar Projections.

## Operation Audit

Operation audit records upload completion and cancellation, folder creation, rename and move, Tag mutations, recycle, restore, Permanent Deletion, quota overrides, and downloads. Lists, Info, File Properties, thumbnails, Inline Preview, and Upload Parts do not create high-frequency operation-audit records.

## Confirmed Defaults

- Managed File Size Limit: 10 GiB.
- Personal Space Quota: 20 GiB, with explicit per-space overrides.
- Upload Part size: 16 MiB unless a Provider requires a larger value.
- Upload Session Inactivity Window: 7 days.
- Trash Retention Period: 30 days.
- Trash purge: daily at 20:00 UTC; upload-session cleanup: daily at 21:00 UTC. Both jobs are seeded, non-deletable, pausable, non-concurrent, and fire once after a misfire.
- Text preview: 1 MiB; image decode/thumbnail ceiling: 40 million pixels; image preview source: 32 MiB; concurrent image jobs: 2.

## Out Of Scope

The first release does not include user-to-user sharing, anonymous links, content full-text indexing, malware scanning, Office conversion, activity/open-history analytics, provider load balancing, provider configuration UI, or user hard deletion.
