GRANT EXECUTE ON FUNCTION public.ensure_system_log_partition(TIMESTAMPTZ) TO PUBLIC;
ALTER FUNCTION public.ensure_system_log_partition(TIMESTAMPTZ) RESET search_path;
ALTER FUNCTION public.ensure_system_log_partition(TIMESTAMPTZ) SECURITY INVOKER;

GRANT EXECUTE ON FUNCTION public.drop_expired_system_log_partition(TEXT, TIMESTAMPTZ) TO PUBLIC;
ALTER FUNCTION public.drop_expired_system_log_partition(TEXT, TIMESTAMPTZ) RESET search_path;
ALTER FUNCTION public.drop_expired_system_log_partition(TEXT, TIMESTAMPTZ) SECURITY INVOKER;
