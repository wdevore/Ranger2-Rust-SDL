use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use geometry::point::Point;
use nodes::{
    node_group::NodeGroup,
    node_properties::{NodeData, RNodeData},
    node_trait::{NodeTrait, NodeType, RNode},
};
use rendering::{color::Palette, render_context::Context, vector_font::VectorFont};
use world::World;

pub struct VectorTextNode {
    data: RNodeData,

    // Hierarchy
    parent: Cell<usize>,

    text: RefCell<String>,
    font: VectorFont,

    // Original model vertices
    vertices: RefCell<Vec<Point>>,

    // Transformed vertices
    bucket: RefCell<Vec<Point>>,
}

impl Drop for VectorTextNode {
    fn drop(&mut self) {
        println!("Dropping: '{}'", self.data().borrow().node.name());
    }
}

impl VectorTextNode {
    pub fn new(name: &str, parent: usize, world: &mut World) -> RNode {
        let mut nd = NodeData::new();
        nd.node.set_name(name.to_string());
        nd.node.set_type(NodeType::Node);
        nd.node.set_id(world.gen_id());

        let n = Self {
            data: Rc::new(RefCell::new(nd)),
            parent: Cell::new(parent),
            vertices: RefCell::new(Vec::new()),
            bucket: RefCell::new(Vec::new()),
            text: RefCell::new(String::from("")),
            font: VectorFont::new(),
        };

        let rc: RNode = Rc::new(RefCell::new(n));

        world.data_mut().add_node(rc.clone());

        NodeGroup::attach_parent(&rc, world.data());

        rc
    }

    pub fn set_text(&self, text: &String) {
        self.text.borrow_mut().replace_range(.., text);

        // Use glyph properties to adjust char location.
        let hoff = self.font.get_horz_offset();
        let mut xpos = 0.0;
        let mut p = Point::new();

        // Rebuild vertex buffer to match text.
        let txt = self.text.borrow();
        let mut v = self.vertices.borrow_mut();
        let mut b = self.bucket.borrow_mut();

        for c in txt.chars() {
            let lines = self.font.get_glyph(c).get_lines();

            for l in lines.iter() {
                p.set_xy(l.x + xpos, l.y);
                v.push(p);
                b.push(Point::new());
            }

            xpos += hoff;
        }

        self.set_node_dirty(true);
    }
}

impl NodeTrait for VectorTextNode {
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
        if self.is_node_dirty() {
            let verts = self.vertices.borrow();
            context.transform(&verts, &self.bucket);
            self.set_node_dirty(false);
        }

        context.set_draw_color(&Palette::WHITE(127));

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
