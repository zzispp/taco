# Materialize personal spaces lazily

Every user conceptually owns a Personal Asset Space keyed by `user_id`, but File Management does not require a cross-context transaction when a user is created. Space metadata is materialized on the first file, folder, or quota mutation; space listings left-join user records so empty users still appear with zero usage. This keeps User and File Management bounded contexts decoupled while preserving one space per user in the domain.
