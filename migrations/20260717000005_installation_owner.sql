CREATE TABLE sys_installation_owner (
    singleton_id SMALLINT NOT NULL DEFAULT 1,
    owner_user_id VARCHAR(36) NOT NULL,
    CONSTRAINT sys_installation_owner_pkey PRIMARY KEY (singleton_id),
    CONSTRAINT chk_sys_installation_owner_singleton CHECK (singleton_id = 1),
    CONSTRAINT uq_sys_installation_owner_owner_user UNIQUE (owner_user_id),
    CONSTRAINT fk_sys_installation_owner_owner_user FOREIGN KEY (owner_user_id) REFERENCES sys_user(user_id) ON DELETE RESTRICT
);

DELETE FROM sys_role
WHERE role_key IN ('admin', 'common');
