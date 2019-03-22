use nodes::{node_nil::NodeNil, node_trait::RNode};

pub enum IOEvent {
    Undefined,
    Mouse,
    Joystick,
    Keyboard,
}

pub struct IOEventData {
    pub event: IOEvent,
    pub coord: (i32, i32),
    pub node: RNode,
}

impl IOEventData {
    pub fn new() -> Self {
        Self {
            event: IOEvent::Undefined,
            coord: (0, 0),
            node: NodeNil::new(),
        }
    }

    pub fn new_mouse_event(x: i32, y: i32) -> Self {
        Self {
            event: IOEvent::Mouse,
            coord: (x, y),
            node: NodeNil::new(),
        }
    }
}
