# Global managed asset library with logical folders

_Status: superseded by ADR 0050._

File Management was initially designed as a single RBAC-controlled Managed Asset Library rather than a personal drive or shared user space. Managed Files kept stable identities, Logical Folders modeled hierarchy independently of storage layout, ordinary files remained private behind authenticated access, and Avatar Assets received an explicit public read projection. This decision was superseded when the product adopted per-user Personal Asset Spaces and Data Scope-based management.
