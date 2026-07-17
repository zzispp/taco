DROP INDEX IF EXISTS idx_sys_system_log_search_ngrams;
DROP INDEX IF EXISTS idx_sys_system_log_ingested_seq;
DROP INDEX IF EXISTS idx_sys_system_log_target_cursor;

CREATE INDEX idx_sys_system_log_target_cursor
    ON sys_system_log (target, occurred_at DESC, id DESC);

ALTER TABLE sys_system_log
    DROP COLUMN search_ngrams,
    DROP COLUMN ingested_seq;

DROP FUNCTION system_log_search_ngrams(TEXT);
