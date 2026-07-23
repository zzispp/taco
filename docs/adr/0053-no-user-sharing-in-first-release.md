# Exclude user-to-user sharing from the first release

Personal Asset Spaces are accessible to their owners and to administrators whose capability permissions intersect the owner's Data Scope; ordinary users do not receive Share Grants in the first release. This keeps one predictable authorization graph, avoids recursive folder ACLs and revocation rules, and leaves sharing as a future bounded-context decision rather than partially implementing the template's invite UI.
