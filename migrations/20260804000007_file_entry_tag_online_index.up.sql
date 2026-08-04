-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_entry_tag_tag_id ON public.file_entry_tag (tag_id);
