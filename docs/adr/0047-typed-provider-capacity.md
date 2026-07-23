# Use typed Provider capacity and dual usage metrics

File Management computes Logical Asset Size, Managed Physical Usage, recycle-bin usage, temporary-upload usage, and deduplication savings from its metadata. A Storage Provider separately reports either bounded capacity with total and available bytes or usage-based capacity without a fabricated total; technical lookup failures remain errors. This keeps the overview accurate for Local disks, quota-bound MinIO buckets, and usage-priced OSS services without using sentinel capacity values.
