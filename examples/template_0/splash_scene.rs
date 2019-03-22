use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

extern crate ranger;

use ranger::{
    nodes::{
        node_properties::{NodeData, RNodeData},
        node_trait::{NodeActions, NodeTrait, NodeType, RNode},
    },
    rendering::{color::Palette, render_context::Context},
    world::{GlobalData, World},
};

pub struct SplashScene {
    replacement: Cell<usize>,

    data: RNodeData,

    parent: Cell<usize>,
}

impl Drop for SplashScene {
    fn drop(&mut self) {
        println!("Dropping: '{}'", self.data().borrow().node.name());
    }
}

impl SplashScene {
    pub fn with_replacement(name: &str, replacement: usize, world: &mut World) -> RNode {
        let mut n = NodeData::new();
        n.node.set_name(name.to_string());
        n.node.set_type(NodeType::Scene);
        n.node.set_id(world.gen_id());
        n.node.make_timing_target(true);

        let ss = Self {
            replacement: Cell::new(replacement),
            data: Rc::new(RefCell::new(n)),
            parent: Cell::new(replacement),
        };

        let rc: RNode = Rc::new(RefCell::new(ss));

        world.data_mut().add_node(rc.clone());

        rc
    }

    pub fn pause_for_seconds(&mut self, pause_for: f64) {
        self.data()
            .borrow_mut()
            .transition
            .set_pause_for(pause_for * 1000.0);
    }
}

impl NodeTrait for SplashScene {
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
    // Timing target
    // --------------------------------------------------------

    fn update(&self, dt: f64) {
        self.data().borrow_mut().transition.update(dt);

        // println!("update '{}', {}", dt, self.to_string());
    }

    // --------------------------------------------------------
    // Transitions
    // --------------------------------------------------------
    fn transition(&self) -> NodeActions {
        // println!("transition '{}'", self.to_string());
        if self.data().borrow().transition.ready() {
            return NodeActions::SceneReplaceTake;
        }

        NodeActions::NoAction
    }

    fn take_transition_node(&self) -> usize {
        self.replacement.replace(0)
    }

    // --------------------------------------------------------
    // Life cycle events
    // --------------------------------------------------------
    fn enter(&self) {
        println!("enter '{}'", self.to_string());
        self.data().borrow_mut().transition.reset_pause();

        // Schedule/enable timing
        self.pause(false); // TODO replace with Ids
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
    // Render events
    // --------------------------------------------------------
    fn visit(&self, context: &mut Context, _interpolation: f64, _gdata: &GlobalData) {
        context.set_draw_color(&Palette::from_hex_rgb(0xffffff));
        context.text(25, 25, "Splash scene", 5, 2);
        // println!("visit '{}'", self.to_string());
    }
}
