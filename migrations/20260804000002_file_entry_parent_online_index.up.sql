-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_entry_parent_id ON public.file_entry (parent_id);
