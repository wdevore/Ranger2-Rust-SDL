extern crate sdl2;

use self::sdl2::render::WindowCanvas;

use std::cell::RefCell;

use events::io_events::{IOEvent, IOEventData};
use nodes::{
    node_nil::NodeNil,
    node_trait::{NodeActions, NodeType, RNode},
};
use rendering::render_context::Context;
use world::GlobalData;

pub struct NodeManager {
    clear_background: bool,

    context: Context,

    // A stack of nodes
    stack: RefCell<NodeStack>,

    timing_targets: RefCell<Vec<RNode>>,
}

impl NodeManager {
    pub fn new(canvas: WindowCanvas, data: &GlobalData) -> Self {
        let mut context = Context::new(canvas);
        context.initialize(data);

        // TODO move clear flag into nodes. each nodes decides if it wants the background
        // cleared verse supplying its own clear.
        Self {
            clear_background: true,
            context: context,
            stack: RefCell::new(NodeStack::new()),
            timing_targets: RefCell::new(Vec::new()),
        }
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn pre_visit(&self) {
        // Typically Scenes/Layers will clear the background themselves so the default
        // is to NOT perform a clear here.
        if self.clear_background {
            // If vsync is enabled then this takes nearly 1/fps milliseconds.
            // For example, 60fps -> 1/60 = ~16.666ms
            self.context.clear();
        }
    }

    pub fn visit(&mut self, interpolation: f64, data: &mut GlobalData) -> bool {
        // Check for scenes
        if self.stack.borrow().is_empty() {
            println!("NodeManager: no more nodes to visit.");
            return false;
        }

        if !self.stack.borrow().next_node_nil() {
            self.set_next_node(data);
        }

        // This will save view-space matrix
        self.context.save();

        // If mouse coords changed then update view coords.
        data.update_view_coords(&mut self.context);

        let mut nodes = self.stack.borrow_mut();
        let action = nodes.running_node().borrow().transition();
        match action {
            NodeActions::SceneReplaceTake => {
                let id = nodes.running_node().borrow().take_transition_node();
                let popped_id = nodes.replace_by_id(id, data);
                // TODO need to make this recursive on children too.
                self.unschedule_timing_target_by_id(popped_id);
            }
            _ => (),
        }

        let rune = nodes.running_node().borrow();
        rune.visit(&mut self.context, interpolation, data);

        // Process view after visiting Nodes.
        self.context.restore();

        true // continue to draw.
    }

    pub fn post_visit(&self) {
        self.context.post();
    }

    pub fn set_next_node(&self, data: &mut GlobalData) {
        let mut nodes = self.stack.borrow_mut();

        if !nodes.next_node_is_transition() {
            if !nodes.running_node_nil() {
                let rune = nodes.running_node().borrow();

                // It is not a transition so it must be a regular scene which means it
                // needs to start transitioning off the stage.
                // Signal node to start exiting the stage via a transition.
                rune.start_exit_transition(data);
                // Transition is complete signal node to complete its exit.
                rune.exit(data);

                // TODO Some nodes may need to release resources
                // if nodes.signal_release() {
                //     // rune.release();
                // }
            }
        };

        // Make the running node the next active node.
        nodes.make_running_node();

        nodes.make_next_node_nil();

        let rune = nodes.running_node().borrow();
        print!("---- Running node ----: ");
        println!("{}", rune.to_string());

        // Are we transitioning from one node to the next.
        if !nodes.running_node_is_transition() {
            // This is a regular node.
            // Signal node that it should enter the stage.
            let rune = nodes.running_node().borrow();
            if !rune.is_nil() {
                rune.enter();
                rune.end_enter_transition();
            }
        }
    }

    pub fn pop_node(&self) {
        self.stack.borrow_mut().pop();
    }

    pub fn push_node(&self, node: RNode) {
        self.stack.borrow_mut().push(node);
    }

    /// Replaces the running node
    // pub fn replace_node(&self, node: RNode) {
    //     {
    //         let stack = self.stack.borrow();
    //         let rune = stack.running_node().borrow();
    //         if rune.is_nil() {
    //             panic!("NodeManager::replace_node -- no running node.");
    //         }
    //     }
    //     self.stack.borrow_mut().replace(node);
    // }

    // --------------------------------------------------------------------------
    // IO events
    // --------------------------------------------------------------------------
    pub fn io_event(&mut self, io_event: IOEventData, data: &mut GlobalData) {
        match io_event.event {
            IOEvent::Mouse => {
                data.set_mouse(io_event.coord.0, io_event.coord.1);

                let mut stack = self.stack.borrow_mut();
                let mut rune = stack.running_node().borrow_mut();
                if !rune.is_nil() {
                    rune.io_event(&io_event);
                }
            }
            _ => (),
        }
    }

    // --------------------------------------------------------------------------
    // Timing
    // --------------------------------------------------------------------------
    pub fn register_timing_target(&self, node: RNode) {
        self.timing_targets.borrow_mut().push(node);
    }

    pub fn unschedule_timing_target(&self, node: RNode) {
        self.timing_targets
            .borrow_mut()
            .retain(|ref n| n.borrow().id() != node.borrow().id());
    }

    pub fn unschedule_timing_target_by_id(&self, node_id: usize) {
        self.timing_targets
            .borrow_mut()
            .retain(|ref n| n.borrow().id() != node_id);
    }

    pub fn update(&self, dt: f64) {
        let targets = self.timing_targets.borrow();
        for target in targets.iter() {
            let t = target.borrow();
            if !t.paused() {
                t.update(dt);
            }
        }
    }
}

// --------------------------------------------------------------------------
// Internal node stack
// --------------------------------------------------------------------------
struct NodeStack {
    nodes: Vec<RNode>,
    // Indicates if a node should dispose completely once it isn't needed
    // anymore. For example, boot and splash scenes typically have this
    // enabled.
    signal_node_to_flush: bool,
    next_node: RNode,
    running_node: RNode,
}

impl Drop for NodeStack {
    fn drop(&mut self) {
        println!("Dropping NodeStack scenes: ({})", self.nodes.len());

        for node in self.nodes.iter() {
            node.borrow().flush(true);
        }

        self.nodes.clear();
    }
}

impl NodeStack {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_node: NodeNil::new(),
            running_node: NodeNil::new(),
            signal_node_to_flush: false,
        }
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn next_node_nil(&self) -> bool {
        let ns = self.next_node.borrow();
        ns.is_nil()
    }

    fn running_node_nil(&self) -> bool {
        let ns = self.running_node.borrow();
        ns.is_nil()
    }

    fn next_node_is_transition(&self) -> bool {
        let ns = self.next_node.borrow();
        ns.is_node_type(NodeType::SceneTransition)
    }

    fn running_node_is_transition(&self) -> bool {
        let ns = self.running_node.borrow();
        ns.is_node_type(NodeType::SceneTransition)
    }

    fn make_next_node_nil(&mut self) {
        self.next_node = NodeNil::new();
    }

    fn running_node(&self) -> &RNode {
        &self.running_node
    }

    fn make_running_node(&mut self) {
        self.running_node = self.next_node.clone();
    }

    // fn signal_flush(&self) -> bool {
    //     self.signal_node_to_flush
    // }

    fn push(&mut self, node: RNode) {
        self.signal_node_to_flush = false;

        self.next_node = node.clone();

        print!("---- Pushing Scene ----: ");
        println!("{}", self.next_node.borrow().to_string());

        self.nodes.push(node);
    }

    fn pop(&mut self) {
        match self.nodes.pop() {
            Some(node) => {
                self.next_node = node;

                // Allow the current running scene a chance to cleanup.
                self.signal_node_to_flush = true;

                print!("---- Popped Scene ----: ");
                println!("{}", self.next_node.borrow().to_string());
            }
            None => {
                // Basically there are no more scenes to execute.
                println!("SceneManager::pop_scene -- no scenes to pop.");
            }
        }
    }

    #[allow(dead_code)]
    fn replace(&mut self, node: RNode, data: &mut GlobalData) {
        self.next_node = node.clone();

        print!("---- With Node ----: ");
        println!("{}", self.next_node.borrow().to_string());

        // println!("Nodes on stack before pop: ({})", self.nodes.len());
        println!("+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==");
        data.print_pool();

        if let Some(pscene) = self.nodes.pop() {
            let pops = pscene.borrow();
            let id = pops.data().borrow().node.id();
            println!("replace has Popped '{}' ({})", pops.to_string(), id);

            // We also need to drop it from the pool

            if let Some(node) = data.take_node(&id) {
                println!("replace flushing '{}'", node.borrow().name());
                node.borrow().flush(true);
            } else {
                println!("replace couldn't take ({}) from pool", id);
            }
        } else {
            println!("replace WARNING nothing popped");
        }

        data.print_pool();
        println!("+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==");

        self.nodes.push(node);

        // println!("Nodes on stack after pop: ({})", self.nodes.len());

        self.signal_node_to_flush = true;
    }

    fn replace_by_id(&mut self, node_id: usize, data: &mut GlobalData) -> usize {
        // println!("Nodes on stack before pop: ({})", self.nodes.len());
        // println!("+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==");
        // data.print_pool();
        let mut popped_id = 0;

        if let Some(pscene) = self.nodes.pop() {
            let pops = pscene.borrow();
            popped_id = pops.data().borrow().node.id();
            println!(
                "replace_by_id has Popped '{}' ({})",
                pops.to_string(),
                popped_id
            );

            // We also need to drop it from the pool
            if let Some(node) = data.take_node(&popped_id) {
                println!("replace_by_id flushing '{}'", node.borrow().name());
                node.borrow().flush(true);
            } else {
                println!("replace_by_id couldn't take ({}) from pool", popped_id);
            }
        } else {
            println!("replace_by_id WARNING nothing popped");
        }

        // data.print_pool();
        // println!("+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==+==");

        if let Some(n) = data.find_node(&node_id) {
            // Make new node next node to run
            self.next_node = n.clone();
            // Place it on the stack
            self.nodes.push((*n).clone());
            print!("---- Next running node ----: ");
            println!("{}", n.borrow().to_string());
        } else {
            println!("replace_by_id couldn't find ({}) from pool", node_id);
        }

        // println!("Nodes on stack after pop: ({})", self.nodes.len());

        self.signal_node_to_flush = true;

        popped_id
    }
}
