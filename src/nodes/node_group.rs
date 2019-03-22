use nodes::node_trait::RNode;
use world::GlobalData;

pub struct NodeGroup;

impl NodeGroup {
    // Take given node, which should have a valid parent assigned,
    // and attach it as a child of the assigned parent.
    pub fn attach_parent(node: &RNode, gdata: &GlobalData) {
        let bnode = node.borrow();
        let op_node_id = bnode.parent();

        let gpart = gdata.find_node(&op_node_id);
        if let Some(parent) = gpart {
            parent.borrow().add_child(node.clone());
        } else {
            println!(
                "attach_parent could not find parent '{}' to attach to.",
                op_node_id
            );
        }
    }
}
