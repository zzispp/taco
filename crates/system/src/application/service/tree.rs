use crate::domain::{Dept, TreeSelectNode};

pub(super) fn dept_tree(depts: Vec<Dept>) -> Vec<TreeSelectNode> {
    depts.iter().filter(|dept| is_root(dept, &depts)).map(|dept| dept_node(dept, &depts)).collect()
}

fn dept_node(dept: &Dept, depts: &[Dept]) -> TreeSelectNode {
    TreeSelectNode {
        id: dept.dept_id.clone(),
        label: dept.dept_name.clone(),
        parent_id: dept.parent_id.clone(),
        disabled: dept.status != constants::system::STATUS_NORMAL,
        children: depts
            .iter()
            .filter(|child| child.parent_id == dept.dept_id)
            .map(|child| dept_node(child, depts))
            .collect(),
    }
}

fn is_root(dept: &Dept, depts: &[Dept]) -> bool {
    dept.parent_id == "0" || !depts.iter().any(|item| item.dept_id == dept.parent_id)
}
