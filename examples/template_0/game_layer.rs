use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ranger::{
    events::io_events::{IOEvent, IOEventData},
    geometry::{aabb::AABBox, point::Point},
    nodes::{
        custom_nodes::transform_filter::TransformFilter,
        node_functions::NodeFunctions,
        node_group::NodeGroup,
        node_properties::{NodeData, RNodeData},
        node_trait::{NodeTrait, NodeType, OChildren, RNode},
    },
    rendering::{color::Palette, render_context::Context, render_context::RenderStyle},
    world::World,
};

use template_0::{orbit_system_node::OrbitSystemNode, rectangle_node::RectangleNode};

// A layer with a background rectangle the same dimensions
// as view-space and with gray color.

pub struct GameLayer {
    data: RNodeData,

    // Hierarchy
    parent: Cell<usize>,

    children: OChildren,

    // Original vertices
    vertices: Vec<Point>,
    // Transformed vertices
    bucket: RefCell<Vec<Point>>,

    background: RefCell<AABBox>,
}

impl Drop for GameLayer {
    fn drop(&mut self) {
        println!("Dropping: '{}'", self.data().borrow().node.name());
    }
}

impl GameLayer {
    pub fn new(name: &str, parent: usize, world: &mut World) -> RNode {
        let mut n = NodeData::new();
        n.node.set_name(name.to_string());
        n.node.set_type(NodeType::Node);
        n.node.set_id(world.gen_id());

        let mut tn = Self {
            data: Rc::new(RefCell::new(n)),
            parent: Cell::new(parent),
            children: Some(RefCell::new(Vec::new())),
            vertices: Vec::new(),
            bucket: RefCell::new(Vec::new()),
            background: RefCell::new(AABBox::new()),
        };

        GameLayer::build(&mut tn, world);

        let rc: RNode = Rc::new(RefCell::new(tn));

        world.data_mut().add_node(rc.clone());

        GameLayer::build_heirarchy(&rc, world);

        rc
    }

    fn build_heirarchy(layer: &RNode, world: &mut World) {
        // Make layer child of its parent (aka GameScene)
        // NodeGroup::attach_parent(layer);
        NodeGroup::attach_parent(layer, world.data());

        let layer_id = NodeFunctions::node_id(layer);

        let filter = TransformFilter::new("GL_OSN_FilterNode", layer_id, world);
        let filter_id = NodeFunctions::node_id(&filter);

        let nrect = OrbitSystemNode::new("OrbitSystemNode", filter_id, world);
        let mut brect = nrect.borrow_mut();
        brect.set_scale(100.0);
        brect.set_position(10.0, -100.0);

        if let Some(n) = brect.as_any_mut().downcast_mut::<OrbitSystemNode>() {
            n.set_color(Palette::ORANGE());
        }

        let filter = TransformFilter::new("GL_YR_FilterNode", layer_id, world);
        let filter_id = NodeFunctions::node_id(&filter);

        let nrect = RectangleNode::new("YellowRect", filter_id, world);
        let mut brect = nrect.borrow_mut();
        brect.set_scale(100.0);
        brect.set_position(100.0, 100.0);

        if let Some(n) = brect.as_any_mut().downcast_mut::<RectangleNode>() {
            n.set_color(Palette::YELLOW());
        }
    }

    fn build(layer: &mut GameLayer, world: &mut World) {
        // Shared edges
        layer.vertices.push(Point::from_xy(-0.5, 0.5));
        layer.vertices.push(Point::from_xy(0.5, 0.5));
        layer.vertices.push(Point::from_xy(0.5, -0.5));
        layer.vertices.push(Point::from_xy(-0.5, -0.5));

        let mut b = layer.bucket.borrow_mut();
        for _ in 0..layer.vertices.len() {
            b.push(Point::new());
        }

        let data = world.data();
        let view_width = data.view_width; // / 2.0;
        let view_height = data.view_height; // / 2.0;

        layer.set_nonuniform_scale(view_width, view_height);
    }
}

impl NodeTrait for GameLayer {
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

        context.set_draw_color(&Palette::DARK_GRAY());

        self.background
            .borrow_mut()
            .set_from_vertices(&self.bucket.borrow());
        context.render_aabb_rectangle(&self.background.borrow(), RenderStyle::Filled);

        // This is slower but allows rotation on the layer
        // context.render_rectangle(&self.bucket);
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
    // IO Events
    // --------------------------------------------------------
    fn io_event(&self, io_event: &IOEventData) {
        match io_event.event {
            IOEvent::Mouse => {
                if let Some(_children) = self.get_children() {}
                // io_event.coord.0, io_event.coord.1
            }
            _ => (),
        }
    }

    // --------------------------------------------------------
    // Grouping
    // --------------------------------------------------------
    fn get_children(&self) -> &OChildren {
        &self.children
    }
}
