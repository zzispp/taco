# Do not migrate legacy avatar URLs

The project has no production deployment or legacy data compatibility requirement, so File Management replaces URL-based avatar references without an importer or dual-read path. Existing development avatars may be discarded, and the new model starts with Managed File identifiers as the only canonical avatar reference. This avoids preserving a Provider-specific URL contract solely for disposable development data.
