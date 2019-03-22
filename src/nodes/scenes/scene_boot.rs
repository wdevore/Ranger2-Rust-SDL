use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nodes::{
    node_properties::{NodeData, RNodeData},
    node_trait::{NodeActions, NodeTrait, NodeType, RNode},
};
use world::World;

pub struct SceneBoot {
    replacement: Cell<usize>,

    data: RNodeData,

    parent: Cell<usize>,
}

impl Drop for SceneBoot {
    fn drop(&mut self) {
        println!("Dropping: '{}'", self.data().borrow().node.name());
    }
}

impl SceneBoot {
    pub fn with_replacement(name: &str, replacement: usize, world: &mut World) -> RNode {
        let mut n = NodeData::new();
        n.node.set_name(name.to_string());
        n.node.set_type(NodeType::Scene);
        n.node.set_id(world.gen_id());

        let sb = Self {
            replacement: Cell::new(replacement),
            data: Rc::new(RefCell::new(n)),
            // parent: Rc::new(RefCell::new(None)),
            parent: Cell::new(replacement),
        };

        let rc: RNode = Rc::new(RefCell::new(sb));

        world.data_mut().add_node(rc.clone());

        rc
    }
}

impl NodeTrait for SceneBoot {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // --------------------------------------------------------
    // Node properties
    // --------------------------------------------------------
    fn data(&self) -> &RNodeData {
        &self.data
    }

    // --------------------------------------------------------
    // Transformations
    // --------------------------------------------------------
    fn parent(&self) -> usize {
        self.parent.get()
    }

    fn set_parent(&self, parent: usize) {
        self.parent.replace(parent);
    }

    // --------------------------------------------------------
    // Life cycle events
    // --------------------------------------------------------
    fn enter(&self) {
        println!("enter '{}'", self.to_string());
    }

    // --------------------------------------------------------
    // Transitions
    // --------------------------------------------------------
    fn transition(&self) -> NodeActions {
        NodeActions::SceneReplaceTake
    }

    fn take_transition_node(&self) -> usize {
        self.replacement.replace(0)
    }

    // fn exit(&self, _data: &GlobalSceneData) {
    //     println!("exit '{}'", self.to_string());
    // }

    // --------------------------------------------------------
    // Rendering
    // --------------------------------------------------------
}
