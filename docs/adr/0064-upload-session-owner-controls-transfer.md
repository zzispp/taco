# Let the upload-session owner control transfer

Only the Upload Session Owner may submit Upload Parts, inspect resumable progress, complete, or voluntarily cancel a session, and every request rechecks capability permission and Data Scope. An administrator with `file:upload:manage` over the target space may inspect or cancel a stuck session but cannot supply bytes or complete it on the owner's behalf. A session identifier never acts as an authorization credential.
