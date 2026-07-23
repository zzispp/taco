CREATE TABLE file_space (
    space_id VARCHAR(36) PRIMARY KEY,
    owner_user_id VARCHAR(36) NOT NULL,
    owner_dept_id VARCHAR(36) NULL,
    quota_override_bytes BIGINT NULL,
    active_bytes BIGINT NOT NULL DEFAULT 0,
    trashed_bytes BIGINT NOT NULL DEFAULT 0,
    reserved_bytes BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    archived_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_file_space_owner FOREIGN KEY (owner_user_id) REFERENCES sys_user(user_id) ON DELETE RESTRICT,
    CONSTRAINT chk_file_space_quota_override CHECK (quota_override_bytes IS NULL OR quota_override_bytes > 0),
    CONSTRAINT chk_file_space_usage CHECK (active_bytes >= 0 AND trashed_bytes >= 0 AND reserved_bytes >= 0),
    CONSTRAINT chk_file_space_status CHECK (status IN ('active', 'archived')),
    CONSTRAINT chk_file_space_archive_time CHECK ((status = 'active' AND archived_at IS NULL) OR (status = 'archived' AND archived_at IS NOT NULL))
);

CREATE UNIQUE INDEX idx_file_space_owner ON file_space (owner_user_id);
CREATE INDEX idx_file_space_status ON file_space (status, updated_at DESC);

CREATE TABLE file_storage_provider (
    provider_key VARCHAR(64) PRIMARY KEY,
    provider_type VARCHAR(64) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT chk_file_storage_provider_key CHECK (BTRIM(provider_key) <> ''),
    CONSTRAINT chk_file_storage_provider_type CHECK (BTRIM(provider_type) <> ''),
    CONSTRAINT chk_file_storage_provider_status CHECK (status IN ('active', 'disabled'))
);

INSERT INTO file_storage_provider (provider_key, provider_type, status, created_at, updated_at)
VALUES ('local', 'local', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

CREATE TABLE file_object (
    object_id VARCHAR(36) PRIMARY KEY,
    provider_key VARCHAR(64) NOT NULL,
    object_key TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256 CHAR(64) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    ref_count BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_file_object_provider FOREIGN KEY (provider_key) REFERENCES file_storage_provider(provider_key) ON DELETE RESTRICT,
    CONSTRAINT chk_file_object_key CHECK (BTRIM(object_key) <> ''),
    CONSTRAINT chk_file_object_size CHECK (size_bytes > 0),
    CONSTRAINT chk_file_object_digest CHECK (sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_file_object_content_type CHECK (BTRIM(content_type) <> ''),
    CONSTRAINT chk_file_object_ref_count CHECK (ref_count >= 0),
    CONSTRAINT chk_file_object_status CHECK (status IN ('active', 'deleting', 'error'))
);

CREATE UNIQUE INDEX idx_file_object_provider_key ON file_object (provider_key, object_key);
CREATE UNIQUE INDEX idx_file_object_digest_size ON file_object (sha256, size_bytes) WHERE status = 'active';
CREATE INDEX idx_file_object_status ON file_object (status, updated_at);

CREATE TABLE file_entry (
    entry_id VARCHAR(36) PRIMARY KEY,
    space_id VARCHAR(36) NOT NULL,
    parent_id VARCHAR(36) NULL,
    kind VARCHAR(16) NOT NULL,
    name VARCHAR(255) NOT NULL,
    normalized_name VARCHAR(255) NOT NULL,
    object_id VARCHAR(36) NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    system_kind VARCHAR(32) NULL,
    trashed_at TIMESTAMPTZ NULL,
    created_by VARCHAR(36) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_by VARCHAR(36) NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_file_entry_space FOREIGN KEY (space_id) REFERENCES file_space(space_id) ON DELETE RESTRICT,
    CONSTRAINT fk_file_entry_parent FOREIGN KEY (parent_id) REFERENCES file_entry(entry_id) ON DELETE RESTRICT,
    CONSTRAINT fk_file_entry_object FOREIGN KEY (object_id) REFERENCES file_object(object_id) ON DELETE RESTRICT,
    CONSTRAINT fk_file_entry_creator FOREIGN KEY (created_by) REFERENCES sys_user(user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_file_entry_updater FOREIGN KEY (updated_by) REFERENCES sys_user(user_id) ON DELETE RESTRICT,
    CONSTRAINT chk_file_entry_kind CHECK (kind IN ('file', 'folder')),
    CONSTRAINT chk_file_entry_name CHECK (BTRIM(name) <> '' AND BTRIM(normalized_name) <> ''),
    CONSTRAINT chk_file_entry_object_kind CHECK ((kind = 'file' AND object_id IS NOT NULL) OR (kind = 'folder' AND object_id IS NULL)),
    CONSTRAINT chk_file_entry_status CHECK (status IN ('active', 'trashed')),
    CONSTRAINT chk_file_entry_system_kind CHECK (system_kind IS NULL OR kind = 'folder'),
    CONSTRAINT chk_file_entry_trash_time CHECK ((status = 'active' AND trashed_at IS NULL) OR (status = 'trashed' AND trashed_at IS NOT NULL)),
    CONSTRAINT chk_file_entry_not_self_parent CHECK (parent_id IS NULL OR parent_id <> entry_id)
);

CREATE UNIQUE INDEX idx_file_entry_active_sibling_name
    ON file_entry (space_id, COALESCE(parent_id, ''), normalized_name)
    WHERE status = 'active';
CREATE UNIQUE INDEX idx_file_entry_avatar_folder
    ON file_entry (space_id)
    WHERE kind = 'folder' AND system_kind = 'avatar';
CREATE INDEX idx_file_entry_parent ON file_entry (space_id, parent_id, status, updated_at DESC);
CREATE INDEX idx_file_entry_created ON file_entry (space_id, created_at DESC);

CREATE TABLE file_tag (
    tag_id VARCHAR(36) PRIMARY KEY,
    space_id VARCHAR(36) NOT NULL,
    name VARCHAR(100) NOT NULL,
    normalized_name VARCHAR(100) NOT NULL,
    created_by VARCHAR(36) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_file_tag_space FOREIGN KEY (space_id) REFERENCES file_space(space_id) ON DELETE CASCADE,
    CONSTRAINT fk_file_tag_creator FOREIGN KEY (created_by) REFERENCES sys_user(user_id) ON DELETE RESTRICT,
    CONSTRAINT chk_file_tag_name CHECK (BTRIM(name) <> '' AND BTRIM(normalized_name) <> '')
);

CREATE UNIQUE INDEX idx_file_tag_space_name ON file_tag (space_id, normalized_name);

CREATE TABLE file_entry_tag (
    entry_id VARCHAR(36) NOT NULL,
    tag_id VARCHAR(36) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (entry_id, tag_id),
    CONSTRAINT fk_file_entry_tag_entry FOREIGN KEY (entry_id) REFERENCES file_entry(entry_id) ON DELETE CASCADE,
    CONSTRAINT fk_file_entry_tag_tag FOREIGN KEY (tag_id) REFERENCES file_tag(tag_id) ON DELETE CASCADE
);

CREATE TABLE file_business_reference (
    reference_id VARCHAR(36) PRIMARY KEY,
    entry_id VARCHAR(36) NOT NULL,
    context_key VARCHAR(100) NOT NULL,
    reference_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_file_business_reference_entry FOREIGN KEY (entry_id) REFERENCES file_entry(entry_id) ON DELETE RESTRICT,
    CONSTRAINT chk_file_business_reference_context CHECK (BTRIM(context_key) <> ''),
    CONSTRAINT chk_file_business_reference_key CHECK (BTRIM(reference_key) <> '')
);

CREATE UNIQUE INDEX idx_file_business_reference_unique ON file_business_reference (context_key, reference_key, entry_id);
CREATE INDEX idx_file_business_reference_entry ON file_business_reference (entry_id);

CREATE TABLE file_upload_session (
    session_id VARCHAR(36) PRIMARY KEY,
    owner_user_id VARCHAR(36) NOT NULL,
    space_id VARCHAR(36) NOT NULL,
    parent_id VARCHAR(36) NULL,
    idempotency_key VARCHAR(255) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    normalized_name VARCHAR(255) NOT NULL,
    declared_size_bytes BIGINT NOT NULL,
    declared_sha256 CHAR(64) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    part_size_bytes BIGINT NOT NULL,
    provider_key VARCHAR(64) NOT NULL,
    provider_upload_ref TEXT NOT NULL,
    provider_object_key TEXT NOT NULL,
    state VARCHAR(16) NOT NULL,
    reserved_bytes BIGINT NOT NULL,
    result_entry_id VARCHAR(36) NULL,
    cleanup_claim_token VARCHAR(36) NULL,
    cleanup_claimed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NULL,
    CONSTRAINT fk_file_upload_owner FOREIGN KEY (owner_user_id) REFERENCES sys_user(user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_file_upload_space FOREIGN KEY (space_id) REFERENCES file_space(space_id) ON DELETE RESTRICT,
    CONSTRAINT fk_file_upload_parent FOREIGN KEY (parent_id) REFERENCES file_entry(entry_id) ON DELETE SET NULL,
    CONSTRAINT fk_file_upload_provider FOREIGN KEY (provider_key) REFERENCES file_storage_provider(provider_key) ON DELETE RESTRICT,
    CONSTRAINT fk_file_upload_result FOREIGN KEY (result_entry_id) REFERENCES file_entry(entry_id) ON DELETE SET NULL,
    CONSTRAINT chk_file_upload_idempotency CHECK (BTRIM(idempotency_key) <> ''),
    CONSTRAINT chk_file_upload_name CHECK (BTRIM(file_name) <> '' AND BTRIM(normalized_name) <> ''),
    CONSTRAINT chk_file_upload_size CHECK (declared_size_bytes > 0 AND part_size_bytes > 0 AND reserved_bytes >= 0),
    CONSTRAINT chk_file_upload_digest CHECK (declared_sha256 IS NULL OR declared_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_file_upload_content_type CHECK (BTRIM(content_type) <> ''),
    CONSTRAINT chk_file_upload_provider_ref CHECK (BTRIM(provider_upload_ref) <> ''),
    CONSTRAINT chk_file_upload_object_key CHECK (BTRIM(provider_object_key) <> ''),
    CONSTRAINT chk_file_upload_cleanup_claim CHECK (
        (cleanup_claim_token IS NULL AND cleanup_claimed_at IS NULL)
        OR (cleanup_claim_token IS NOT NULL AND cleanup_claimed_at IS NOT NULL)
    ),
    CONSTRAINT chk_file_upload_state CHECK (state IN ('open', 'completing', 'completed', 'aborted', 'expired')),
    CONSTRAINT chk_file_upload_lifecycle CHECK (
        (state IN ('open', 'completing') AND result_entry_id IS NULL AND completed_at IS NULL AND reserved_bytes = declared_size_bytes)
        OR (state = 'completed' AND completed_at IS NOT NULL AND reserved_bytes = 0)
        OR (state IN ('aborted', 'expired') AND result_entry_id IS NULL AND completed_at IS NOT NULL AND reserved_bytes = 0)
    )
);

CREATE UNIQUE INDEX idx_file_upload_intent ON file_upload_session (owner_user_id, space_id, idempotency_key);
CREATE INDEX idx_file_upload_cleanup ON file_upload_session (state, last_activity_at) WHERE cleanup_claim_token IS NULL;
CREATE INDEX idx_file_upload_space_state ON file_upload_session (space_id, state, created_at DESC);

CREATE TABLE file_provider_cleanup (
    cleanup_id VARCHAR(36) PRIMARY KEY,
    provider_key VARCHAR(64) NOT NULL,
    cleanup_kind VARCHAR(16) NOT NULL,
    object_key TEXT NULL,
    upload_ref TEXT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    attempt_count BIGINT NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL,
    claim_token VARCHAR(36) NULL,
    claimed_at TIMESTAMPTZ NULL,
    last_error_code VARCHAR(128) NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_file_provider_cleanup_provider FOREIGN KEY (provider_key) REFERENCES file_storage_provider(provider_key) ON DELETE RESTRICT,
    CONSTRAINT chk_file_provider_cleanup_kind CHECK (
        (cleanup_kind = 'object' AND object_key IS NOT NULL AND BTRIM(object_key) <> '' AND upload_ref IS NULL)
        OR (cleanup_kind = 'upload' AND object_key IS NULL AND upload_ref IS NOT NULL AND BTRIM(upload_ref) <> '')
    ),
    CONSTRAINT chk_file_provider_cleanup_status CHECK (status IN ('pending', 'deleting', 'done', 'error')),
    CONSTRAINT chk_file_provider_cleanup_attempts CHECK (attempt_count >= 0),
    CONSTRAINT chk_file_provider_cleanup_claim CHECK (
        (claim_token IS NULL AND claimed_at IS NULL)
        OR (claim_token IS NOT NULL AND claimed_at IS NOT NULL)
    )
);

CREATE UNIQUE INDEX idx_file_provider_cleanup_identity
    ON file_provider_cleanup (provider_key, cleanup_kind, COALESCE(object_key, ''), COALESCE(upload_ref, ''))
    WHERE status <> 'done';
CREATE INDEX idx_file_provider_cleanup_queue
    ON file_provider_cleanup (status, next_attempt_at, updated_at)
    WHERE status IN ('pending', 'error');

CREATE TABLE file_upload_part (
    session_id VARCHAR(36) NOT NULL,
    part_number BIGINT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256 CHAR(64) NOT NULL,
    provider_part_ref TEXT NULL,
    state VARCHAR(16) NOT NULL DEFAULT 'completed',
    claim_token VARCHAR(36) NULL,
    claimed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (session_id, part_number),
    CONSTRAINT fk_file_upload_part_session FOREIGN KEY (session_id) REFERENCES file_upload_session(session_id) ON DELETE CASCADE,
    CONSTRAINT chk_file_upload_part_number CHECK (part_number > 0),
    CONSTRAINT chk_file_upload_part_size CHECK (size_bytes > 0),
    CONSTRAINT chk_file_upload_part_digest CHECK (sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_file_upload_part_state CHECK (state IN ('writing', 'completed')),
    CONSTRAINT chk_file_upload_part_claim CHECK (
        (state = 'writing' AND provider_part_ref IS NULL AND claim_token IS NOT NULL AND claimed_at IS NOT NULL)
        OR (state = 'completed' AND provider_part_ref IS NOT NULL AND BTRIM(provider_part_ref) <> '' AND claim_token IS NULL AND claimed_at IS NULL)
    )
);

CREATE FUNCTION archive_user_file_space() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.del_flag <> '2' AND NEW.del_flag = '2' THEN
        UPDATE file_space
        SET owner_dept_id = NEW.dept_id,
            status = 'archived',
            archived_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE owner_user_id = NEW.user_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_archive_user_file_space
    AFTER UPDATE OF del_flag ON sys_user
    FOR EACH ROW EXECUTE FUNCTION archive_user_file_space();
