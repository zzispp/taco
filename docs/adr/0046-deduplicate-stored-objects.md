# Deduplicate immutable stored objects

_Status: superseded by ADR 0051._

The initial design allowed any caller to bypass upload when a globally matching SHA-256 Stored Object existed. That shortcut was superseded after adopting Personal Asset Spaces because a digest-only lookup across inaccessible spaces would disclose private-content existence and enable unauthorized references.
