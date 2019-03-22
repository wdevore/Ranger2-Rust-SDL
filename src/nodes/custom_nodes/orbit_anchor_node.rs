use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use animation::motion::AngularMotion;
use nodes::{
    node_group::NodeGroup,
    node_properties::{NodeData, RNodeData},
    node_trait::{NodeTrait, NodeType, OChildren, RNode},
};
use world::World;

// An anchor node is a headless node used for various types of "associated"
// transformations.
// It will modify Context.current prior to any children being visited.
//
// The anchor needs only the translation from the parent in order to sync
// its position. It will then add a rotation which is then passed to the
// children.

pub struct OrbitAnchorNode {
    data: RNodeData,

    children: OChildren,

    // Hierarchy
    parent: Cell<usize>,

    angle_motion: RefCell<AngularMotion>,
}

impl Drop for OrbitAnchorNode {
    fn drop(&mut self) {
        println!("Dropping: '{}'", self.data().borrow().node.name());
    }
}

impl OrbitAnchorNode {
    pub fn new(name: &str, parent: usize, world: &mut World) -> RNode {
        let mut n = NodeData::new();
        n.node.set_name(name.to_string());
        n.node.set_type(NodeType::Node);
        n.node.set_id(world.gen_id());
        n.node.make_timing_target(true);

        let an = Self {
            data: Rc::new(RefCell::new(n)),
            parent: Cell::new(parent),
            children: Some(RefCell::new(Vec::new())),
            angle_motion: RefCell::new(AngularMotion::new()),
        };

        OrbitAnchorNode::construct(&an, world);

        let rc: RNode = Rc::new(RefCell::new(an));

        world.data_mut().add_node(rc.clone());

        NodeGroup::attach_parent(&rc, world.data());

        rc
    }

    fn construct(node: &OrbitAnchorNode, _world: &mut World) {
        node.angle_motion.borrow_mut().set_step_value(-5.0);
        // node.set_rotation_degrees(-15.0);
    }
}

impl NodeTrait for OrbitAnchorNode {
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
    // Timing target
    // --------------------------------------------------------
    fn update(&self, dt: f64) {
        self.angle_motion.borrow_mut().update(dt);
    }

    // --------------------------------------------------------
    // Rendering: visiting and drawing
    // --------------------------------------------------------
    fn interpolate(&self, interpolation: f64) {
        let value = self.angle_motion.borrow_mut().interpolate(interpolation);
        self.set_rotation_degrees(value);
    }

    // --------------------------------------------------------
    // Grouping
    // --------------------------------------------------------
    fn get_children(&self) -> &OChildren {
        &self.children
    }
}
