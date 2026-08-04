-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_upload_session_result_entry_id ON public.file_upload_session (result_entry_id);
