/// Permission expression attached to both a route and its handler declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionRequirement {
    AllOf(&'static [&'static str]),
    AnyOf(&'static [&'static str]),
}

impl PermissionRequirement {
    pub const fn all_of(permissions: &'static [&'static str]) -> Self {
        Self::AllOf(permissions)
    }

    pub const fn any_of(permissions: &'static [&'static str]) -> Self {
        Self::AnyOf(permissions)
    }

    pub fn is_satisfied_by(self, granted: &[String]) -> bool {
        let contains = |required: &&str| granted.iter().any(|item| item == required);
        match self {
            Self::AllOf(required) => !required.is_empty() && required.iter().all(contains),
            Self::AnyOf(required) => required.iter().any(contains),
        }
    }

    pub fn is_equivalent_to(self, other: Self) -> bool {
        match (self, other) {
            (Self::AllOf(left), Self::AllOf(right)) | (Self::AnyOf(left), Self::AnyOf(right)) => same_permissions(left, right),
            (Self::AllOf(_), Self::AnyOf(_)) | (Self::AnyOf(_), Self::AllOf(_)) => false,
        }
    }
}

fn same_permissions(left: &[&str], right: &[&str]) -> bool {
    left.len() == right.len() && left.iter().all(|permission| right.contains(permission)) && right.iter().all(|permission| left.contains(permission))
}

/// Compile-time handler metadata collected for startup validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtectedHandler {
    pub function: &'static str,
    pub requirement: PermissionRequirement,
}

inventory::collect!(ProtectedHandler);

/// Authorization requirement for one HTTP route pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutePermissionRule {
    pub methods: Vec<String>,
    pub path_pattern: String,
    pub requirement: PermissionRequirement,
    pub handler: &'static str,
}

#[cfg(test)]
mod tests {
    use super::PermissionRequirement;

    #[rbac_macros::require_any_perms("job:import", "job:edit")]
    fn any_permission_handler() {}

    #[test]
    fn permission_requirements_enforce_all_and_any_semantics() {
        let granted = vec!["job:import".into(), "job:read".into()];

        assert!(PermissionRequirement::all_of(&["job:import", "job:read"]).is_satisfied_by(&granted));
        assert!(!PermissionRequirement::all_of(&["job:import", "job:edit"]).is_satisfied_by(&granted));
        assert!(PermissionRequirement::any_of(&["job:import", "job:edit"]).is_satisfied_by(&granted));
        assert!(!PermissionRequirement::any_of(&["job:edit", "job:remove"]).is_satisfied_by(&granted));
        assert!(!PermissionRequirement::all_of(&[]).is_satisfied_by(&granted));
        assert!(!PermissionRequirement::any_of(&[]).is_satisfied_by(&granted));
        assert!(PermissionRequirement::any_of(&["job:import", "job:edit"]).is_equivalent_to(PermissionRequirement::any_of(&["job:edit", "job:import"])));
        assert!(!PermissionRequirement::all_of(&["job:import"]).is_equivalent_to(PermissionRequirement::any_of(&["job:import"])));
    }

    #[test]
    fn require_any_perms_registers_the_complete_requirement() {
        any_permission_handler();
        let registered = inventory::iter::<super::ProtectedHandler>
            .into_iter()
            .find(|handler| handler.function == "any_permission_handler")
            .unwrap();

        assert_eq!(registered.requirement, PermissionRequirement::any_of(&["job:import", "job:edit"]));
    }
}
