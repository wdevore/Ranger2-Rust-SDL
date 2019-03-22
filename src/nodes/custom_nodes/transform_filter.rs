use std::any::Any;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use math::affine_transform::AffineTransform;
use nodes::{
    node_functions::NodeFunctions,
    node_group::NodeGroup,
    node_properties::{NodeData, RNodeData},
    node_trait::{NodeTrait, NodeType, OChildren, RNode},
};
use rendering::render_context::Context;
use world::{GlobalData, World};

pub struct TransformFilter {
    data: RNodeData,

    children: OChildren,

    // Hierarchy
    parent: Cell<usize>,

    // Filters
    exclude_translation: bool,
    exclude_rotation: bool,
    exclude_scale: bool,
}

impl Drop for TransformFilter {
    fn drop(&mut self) {
        println!("Dropping: '{}'", self.data().borrow().node.name());
    }
}

impl TransformFilter {
    pub fn new(name: &str, parent_id: usize, world: &mut World) -> RNode {
        let mut n = NodeData::new();
        n.node.set_name(name.to_string());
        n.node.set_type(NodeType::Node);
        n.node.set_id(world.gen_id());

        let an = Self {
            data: Rc::new(RefCell::new(n)),
            parent: Cell::new(parent_id),
            children: Some(RefCell::new(Vec::new())),
            // By default most nodes will want to "inherit" the parent translation
            // thus we don't want to exclude it.
            exclude_translation: false,
            // Most nodes will NOT want any rotation or scale from the parent
            // thus we wan't to exclude them.
            exclude_rotation: true,
            exclude_scale: true,
        };

        let rc: RNode = Rc::new(RefCell::new(an));

        world.data_mut().add_node(rc.clone());

        TransformFilter::construct(&rc, world);

        rc
    }

    fn construct(node: &RNode, world: &mut World) {
        NodeGroup::attach_parent(node, world.data());
    }

    pub fn exclude_translation(&mut self, exclude: bool) {
        self.exclude_translation = exclude;
    }

    pub fn exclude_rotation(&mut self, exclude: bool) {
        self.exclude_rotation = exclude;
    }

    pub fn exclude_scale(&mut self, exclude: bool) {
        self.exclude_scale = exclude;
    }
}

impl NodeTrait for TransformFilter {
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
    // Rendering: visiting and drawing
    // --------------------------------------------------------
    fn visit(&self, context: &mut Context, interpolation: f64, gdata: &GlobalData) {
        context.save();

        if let Some(children) = self.get_children() {
            // Visit any children contained by this node.
            for child in children.borrow().iter() {
                context.save();

                // TODO Figure out a way to cache `inv` and `components`
                let parent_id = self.parent();
                if parent_id != 0 {
                    let data = NodeFunctions::get_rnode_data(parent_id, gdata);
                    {
                        let bdata = data.borrow();
                        let inv = bdata.transform.get_inverse();

                        context.apply(&inv);
                    }

                    let mut components = AffineTransform::new();

                    // Now re-introduce just specific components from the parent
                    data.borrow_mut().transform.calc_filtered_transform(
                        self.exclude_translation,
                        self.exclude_rotation,
                        self.exclude_scale,
                        &mut components,
                    );

                    // println!("filter: components: {:?}", components);
                    context.apply(&components);
                } else {
                    dbg!("Parent NOT FOUND");
                    return;
                }

                child.borrow().visit(context, interpolation, gdata);

                context.restore();
            }
        }

        context.restore();
    }

    // --------------------------------------------------------
    // Grouping
    // --------------------------------------------------------
    fn get_children(&self) -> &OChildren {
        &self.children
    }
}
