# Keep Storage Providers limited to opaque object I/O

A Storage Provider streams multipart writes, completion, reads and byte ranges, deletion, object statistics, and typed capacity for opaque object keys. It never owns Logical Folders, Tags, avatar rules, preview policy, authorization, or public URLs. File Management infrastructure performs content detection and derivatives and exposes stable authenticated APIs, so Local, OSS, and MinIO adapters implement one narrow storage contract without duplicating business behavior.
