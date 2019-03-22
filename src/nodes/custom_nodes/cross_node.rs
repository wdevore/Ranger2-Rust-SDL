use std::any::Any;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use geometry::point::Point;
use nodes::{
    node_group::NodeGroup,
    node_properties::{NodeData, RNodeData},
    node_trait::{NodeTrait, NodeType, RNode},
};
use rendering::{color::Palette, render_context::Context};
use world::World;

// A basic leaf node that renders a "+"

pub struct CrossNode {
    data: RNodeData,

    // Hierarchy
    parent: Cell<usize>,

    // Original model vertices
    vertices: Vec<Point>,
    // Transformed vertices
    bucket: RefCell<Vec<Point>>,
}

impl Drop for CrossNode {
    fn drop(&mut self) {
        println!("Dropping: '{}'", self.data().borrow().node.name());
    }
}

impl CrossNode {
    pub fn new(name: &str, parent: usize, world: &mut World) -> RNode {
        let mut n = NodeData::new();
        n.node.set_name(name.to_string());
        n.node.set_type(NodeType::Node);
        n.node.set_id(world.gen_id());

        let mut tn = Self {
            data: Rc::new(RefCell::new(n)),
            // parent: Rc::new(RefCell::new(parent)),
            parent: Cell::new(parent),
            vertices: Vec::new(),
            bucket: RefCell::new(Vec::new()),
        };

        CrossNode::construct(&mut tn);

        let rc: RNode = Rc::new(RefCell::new(tn));

        world.data_mut().add_node(rc.clone());

        NodeGroup::attach_parent(&rc, world.data());

        rc
    }

    fn construct(node: &mut CrossNode) {
        // Horizontal
        node.vertices.push(Point::from_xy(-0.5, 0.0));
        node.vertices.push(Point::from_xy(0.5, 0.0));

        // Vertical
        node.vertices.push(Point::from_xy(0.0, -0.5));
        node.vertices.push(Point::from_xy(0.0, 0.5));

        let mut b = node.bucket.borrow_mut();
        for _ in 0..node.vertices.len() {
            b.push(Point::new());
        }
    }
}

impl NodeTrait for CrossNode {
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
    // Rendering
    // --------------------------------------------------------
    fn draw(&self, context: &mut Context) {
        // Transform this node's vertices using the context
        if self.is_node_dirty() {
            context.transform(&self.vertices, &self.bucket);
            self.set_node_dirty(false);
        }

        context.set_draw_color(&Palette::WHITE(255));

        context.render_lines(&self.bucket);
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
}
