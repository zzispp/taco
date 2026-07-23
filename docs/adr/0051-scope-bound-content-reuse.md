# Bound hash-only content reuse by visibility scope

An Upload Session may skip part transfer only when its declared SHA-256 Content Digest matches a verified Stored Object inside the caller's Content Reuse Scope: the caller's own space or a Data Scope-visible space. A digest match outside that scope never reveals existence or grants a reference; the caller must provide Proof of Possession by uploading and validating the bytes. After proof, the immutable Stored Object may still be physically deduplicated across spaces, with each Managed File retaining its own authorization boundary.
