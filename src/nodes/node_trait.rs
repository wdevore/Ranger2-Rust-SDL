use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use events::io_events::IOEventData;
use math::affine_transform::AffineTransform;
use nodes::{node_functions::NodeFunctions, node_properties::RNodeData};

use rendering::render_context::Context;
use world::GlobalData;

// The node system is similar to Inventor and/or Cocos2D:
// http://www-evasion.imag.fr/~Francois.Faure/doc/inventorMentor/sgi_html/ch09.html

pub enum NodeActions {
    NoAction,
    SceneReplace,
    SceneReplaceTake,
    SceneReplaceTakeUnRegister,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeType {
    Nil,
    Node,
    Group,
    Scene,
    SceneTransition,
}

pub type RNode = Rc<RefCell<NodeTrait>>;

pub type OChildren = Option<RefCell<Vec<RNode>>>;

// This trait will always be contained in an Rc<RefCell<>>
pub trait NodeTrait {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // --------------------------------------------------------
    // PartialEq and container delegates
    // --------------------------------------------------------
    // Typically Scheduler uses this indirectly via PartialEq checks.
    fn tt_eq(&self, other: &NodeTrait) -> bool {
        self.id() == other.id()
    }

    // --------------------------------------------------------
    // Node properties
    // --------------------------------------------------------
    fn data(&self) -> &RNodeData;
    // fn data_mut(&mut self) -> &mut NodeData;

    fn id(&self) -> usize {
        self.data().borrow().node.id()
    }
    fn set_id(&self, id: usize) {
        self.data().borrow_mut().node.set_id(id);
    }

    fn name(&self) -> String {
        self.data().borrow().node.name().to_string()
    }

    fn set_name(&self, name: String) {
        self.data().borrow_mut().node.set_name(name);
    }

    fn get_node_type(&self) -> NodeType {
        self.data().borrow().node.node_type()
    }

    fn is_node_type(&self, n_type: NodeType) -> bool {
        self.data().borrow().node.node_type() == n_type
    }

    fn is_nil(&self) -> bool {
        self.get_node_type() == NodeType::Nil
    }

    fn is_visible(&self) -> bool {
        self.data().borrow().node.visible()
    }

    // --------------------------------------------------------
    // Rendering: visiting, modification and drawing
    // --------------------------------------------------------
    // visit() may modify Context's current transform prior to
    // draw() being called.
    // `interpolation` should only be applied to deltas, for example,
    // velocities/direction-vectors or angular velocities.
    // Update() will modify velocities which the interpolation would act against.
    fn visit(&self, context: &mut Context, interpolation: f64, gdata: &GlobalData) {
        // println!(
        //     "visiting --------------------- {}, stack: {}",
        //     self.name(),
        //     context.top_index()
        // );
        if !self.is_visible() {
            return;
        }

        // println!("Stack as saved at: ({})", context.top_index());
        context.save();
        // context.print_stack(10);

        // Because position and angles are dependent
        // on lerping we perform interpolation first.
        self.interpolate(interpolation);

        // We need to scope the data() here because the draw() method will
        // also want to borrow data().
        {
            let mut data = self.data().borrow_mut();
            let aft: &AffineTransform;
            if data.node.is_dirty() {
                aft = data.transform.calc_transform();
            } else {
                aft = data.transform.get_transform();
            }

            context.apply(aft);
            // println!("context.applied : {:?}", context.current());
            // context.print_stack(10);
        }

        if let Some(children) = self.get_children() {
            // println!("Drawing parent '{}'", self.name());
            self.draw(context); // Draw parent

            // Visit any children contained by this node.
            for child in children.borrow().iter() {
                // println!(
                //     "visiting child '{}' of '{}'",
                //     child.borrow().name(),
                //     self.name()
                // );
                child.borrow().visit(context, interpolation, gdata);
                // println!("Done visiting child '{}'", child.borrow().name());
            }
        } else {
            // Just draw node
            // println!("Drawing leaf '{}'", self.name());
            self.draw(context);
        }

        context.restore();
        // context.print_stack(10);

        // Do any post rendering. Note this is really for debugging purposes only.
        // self.device_visit(context);
    }

    // visit() calls this method
    fn draw(&self, &mut Context) {
        // println!("{} has no rendering.", self.name());
    }

    fn interpolate(&self, _interpolation: f64) {}

    // --------------------------------------------------------
    // Grouping
    // --------------------------------------------------------
    fn get_children(&self) -> &OChildren {
        &None
    }

    fn add_child(&self, node: RNode) {
        if let Some(children) = self.get_children() {
            children.borrow_mut().push(node.clone());
        }
    }

    // --------------------------------------------------------
    // Transformations
    // --------------------------------------------------------
    fn set_position(&self, x: f64, y: f64) {
        self.data().borrow_mut().transform.set_position(x, y);
        self.ripple_node_dirty(true);
    }

    fn set_rotation_degrees(&self, degrees: f64) {
        self.data()
            .borrow_mut()
            .transform
            .set_rotation_degrees(degrees);
        self.ripple_node_dirty(true);
    }

    fn set_scale(&self, s: f64) {
        self.data().borrow_mut().transform.set_scale(s);
        self.ripple_node_dirty(true);
    }

    fn set_nonuniform_scale(&self, sx: f64, sy: f64) {
        self.data()
            .borrow_mut()
            .transform
            .set_nonuniform_scale(sx, sy);
        self.ripple_node_dirty(true);
    }

    fn parent(&self) -> usize {
        0
    }
    fn set_parent(&self, _parent: usize) {}

    // ^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==
    // Dirty state
    // ^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==^==
    fn set_node_dirty(&self, dirty: bool) {
        self.data().borrow_mut().node.set_dirty(dirty);
    }

    fn is_node_dirty(&self) -> bool {
        self.data().borrow().node.is_dirty()
    }

    fn ripple_node_dirty(&self, dirty: bool) {
        if let Some(children) = self.get_children() {
            for child in children.borrow().iter() {
                child.borrow_mut().ripple_node_dirty(dirty);
            }
        }
        self.data().borrow_mut().node.set_dirty(dirty);
    }

    // --------------------------------------------------------
    // IO Events
    // --------------------------------------------------------
    fn io_event(&self, io_event: &IOEventData) {
        if let Some(children) = self.get_children() {
            for child in children.borrow().iter() {
                println!("io_event '{}' of '{}'", child.borrow().name(), self.name());
                child.borrow_mut().io_event(io_event);
            }
        }
    }

    // --------------------------------------------------------
    // Mappings
    // --------------------------------------------------------
    // TODO add psuedo_root: RNode
    fn node_to_world(&mut self, to_world: &mut AffineTransform, gdata: &GlobalData) {
        let mut data = self.data().borrow_mut();

        // A composite transform to accumulate the parent transforms.
        let compo_aft = data.transform.calc_transform(); // Start with this child

        // Use a copy
        let mut comp = AffineTransform::from_transform(compo_aft);

        // Iterate "upwards" starting with this child's parent.
        let mut ro_parent = self.parent();
        if ro_parent == 0 {
            *to_world = comp;
            return;
        }

        let mut pre = AffineTransform::new();

        'climb: loop {
            let pdata = NodeFunctions::get_rnode_data(ro_parent, gdata);
            let mut bdata = pdata.borrow_mut();
            let parent_aft = bdata.transform.calc_transform();

            // Because we are iterating upwards we need to pre-multiply each
            // child. Ex: [child] x [parent_aft]
            //
            // ----------------------------------------------------------
            //           [compo] x [parent_aft] = pre
            //                   |
            //                   v
            //                 [compo] x [parent_aft]
            //                         |
            //                         v
            //                      [compo] x [parent_aft...]
            //
            // This is a pre-multiply order
            // [child] x [parent ofchild] x [parent of parent of child]...
            //
            // In other words the child is mutiplied "into" the parent.

            AffineTransform::multiply_mn(&comp, parent_aft, &mut pre);
            comp = pre;

            // if child == psuedo_root {
            //     break;
            // }

            // Move upwards to next parent
            let gpart = gdata.find_node(&ro_parent);
            ro_parent = gpart.unwrap().borrow().parent();

            if ro_parent == 0 {
                break 'climb;
            }
        }

        // Copy to output: world
        *to_world = comp;
    }

    // --------------------------------------------------------
    // Life cycle events
    // --------------------------------------------------------
    fn start_exit_transition(&self, &mut GlobalData) {}
    fn end_enter_transition(&self) {}

    // A leaf node will override this.
    fn enter(&self) {}
    fn sub_enter(&self, children: &RefCell<Vec<RNode>>) {
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                child.borrow().enter();
                self.sub_enter(sub_children);
            } else {
                child.borrow().enter();
            }
        }
    }

    // A leaf node will override this.
    fn exit(&self, data: &mut GlobalData) {
        if let Some(children) = self.get_children() {
            self.sub_exit(data, children);
        }
    }
    fn sub_exit(&self, data: &mut GlobalData, children: &RefCell<Vec<RNode>>) {
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                child.borrow().exit(data);
                self.sub_exit(data, sub_children);
            } else {
                child.borrow().exit(data);
            }
        }
    }

    fn flush(&self, flush: bool) {
        if let Some(children) = self.get_children() {
            // println!(
            //     "Flushing children of {} ({})",
            //     self.name(),
            //     children.borrow().len()
            // );
            self.sub_flush(flush, children);
        } else {
            // TODO perhaps unregister for stuff like io events and timing targets

        }
    }

    fn sub_flush(&self, flush: bool, children: &RefCell<Vec<RNode>>) {
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                // let parent = child.borrow().parent();
                // if let Some(part) = parent.borrow().as_ref() {
                //     data.unregister_for_io_events(*part, child.borrow().data().borrow().node.id());
                // }
                child.borrow().flush(flush);
                self.sub_flush(flush, sub_children);
            } else {
                // let parent = child.borrow().parent();
                // if let Some(part) = parent.borrow().as_ref() {
                //     data.unregister_for_io_events(*part, child.borrow().data().borrow().node.id());
                // }
                child.borrow().flush(flush);
            }
        }

        children.borrow_mut().clear();
    }

    // --------------------------------------------------------
    // Timing target
    // --------------------------------------------------------
    fn update(&self, _dt: f64) {}

    fn pause(&self, paused: bool) {
        self.data().borrow_mut().timing.pause(paused);
    }

    fn paused(&self) -> bool {
        self.data().borrow().timing.paused()
    }

    fn ripple_pause(&self, paused: bool) {
        println!("Ripple pausing: {} for '{}'", paused, self.name());
        if let Some(children) = self.get_children() {
            println!("Pausing children of {}", self.name());
            self.sub_ripple_pause(paused, children);
        }
    }

    fn sub_ripple_pause(&self, paused: bool, children: &RefCell<Vec<RNode>>) {
        // let mut sub_name = String::from("__");
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                child.borrow().pause(paused);
                // sub_name = child.borrow().name();
                // println!("recursing sub pause children: {}", sub_name);
                self.sub_ripple_pause(paused, sub_children);
            } else {
                child.borrow().pause(paused);
            }
        }
        // println!("Pause bubbling up from: {}", sub_name);
    }

    // --------------------------------------------------------
    // Transitions
    // --------------------------------------------------------
    fn transition(&self) -> NodeActions {
        NodeActions::NoAction
    }

    fn take_transition_node(&self) -> usize {
        0
    }

    fn get_transition_node(&self) -> usize {
        0
    }

    // --------------------------------------------------------
    // Misc
    // --------------------------------------------------------
    fn to_string(&self) -> String {
        self.name()
    }
}

impl PartialEq for NodeTrait {
    fn eq(&self, other: &NodeTrait) -> bool {
        self.tt_eq(other)
    }
}
