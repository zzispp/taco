CREATE TABLE sys_user_session (
    token_id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    dept_name VARCHAR(30) NULL,
    user_name VARCHAR(30) NOT NULL,
    ipaddr VARCHAR(128) NOT NULL,
    login_location VARCHAR(255) NOT NULL,
    browser VARCHAR(128) NOT NULL,
    os VARCHAR(128) NOT NULL,
    login_time TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_sys_user_session_user FOREIGN KEY (user_id) REFERENCES sys_user(user_id) ON DELETE CASCADE,
    CONSTRAINT chk_sys_user_session_token_id CHECK (BTRIM(token_id) <> ''),
    CONSTRAINT chk_sys_user_session_expiry CHECK (expires_at > login_time)
);

CREATE INDEX idx_sys_user_session_user ON sys_user_session (user_id);
CREATE INDEX idx_sys_user_session_expires_at ON sys_user_session (expires_at);
