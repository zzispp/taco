-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sys_user_avatar_file_id ON public.sys_user (avatar_file_id);
