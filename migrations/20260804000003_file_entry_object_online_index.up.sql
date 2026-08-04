-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_entry_object_id ON public.file_entry (object_id);
