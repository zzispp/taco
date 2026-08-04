ALTER FUNCTION public.ensure_system_log_partition(TIMESTAMPTZ) SECURITY DEFINER;
ALTER FUNCTION public.ensure_system_log_partition(TIMESTAMPTZ) SET search_path TO pg_catalog, public;
REVOKE ALL ON FUNCTION public.ensure_system_log_partition(TIMESTAMPTZ) FROM PUBLIC;

ALTER FUNCTION public.drop_expired_system_log_partition(TEXT, TIMESTAMPTZ) SECURITY DEFINER;
ALTER FUNCTION public.drop_expired_system_log_partition(TEXT, TIMESTAMPTZ) SET search_path TO pg_catalog, public;
REVOKE ALL ON FUNCTION public.drop_expired_system_log_partition(TEXT, TIMESTAMPTZ) FROM PUBLIC;
