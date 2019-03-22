use std::cell::RefCell;

use world::GlobalData;

use geometry::{aabb::AABBox, point::Point};
use math::affine_transform::AffineTransform;
use nodes::{
    node_manager::NodeManager, node_nil::NodeNil, node_properties::RNodeData, node_trait::RNode,
};
use rendering::{color::Palette, render_context::Context, render_context::RenderStyle};

pub struct NodeFunctions;

impl NodeFunctions {
    // ------------------------------------------------------------
    // Drawing / Rendering
    // ------------------------------------------------------------
    pub fn draw_aabb(vertices: &Vec<Point>, context: &mut Context) {
        let mut aabb = AABBox::new();
        aabb.set_from_vertices(vertices);
        context.set_draw_color(&Palette::RED());
        context.render_aabb_rectangle(&aabb, RenderStyle::Outline);
    }

    // ------------------------------------------------------------
    // Space mapping
    // ------------------------------------------------------------
    // Map device/mouse/pixel/window space to view-space.
    pub fn map_device_to_view(dx: i32, dy: i32, context: &mut Context) -> (f64, f64) {
        let inv = context.get_view_space().inverse();
        let device = Point::from_xy(dx as f64, dy as f64);
        let mut view = Point::new();
        AffineTransform::transform_to_point(&device, &mut view, &inv);
        (view.x, view.y)
    }

    // Note: world is the identity matrix so view is actually used
    pub fn map_device_to_node(
        dx: i32,
        dy: i32,
        node: &RNode,
        context: &mut Context,
        gdata: &GlobalData,
    ) -> (f64, f64) {
        let dev_map = NodeFunctions::map_device_to_view(dx, dy, context);
        let device = Point::from_tup(dev_map);

        let mut aft = AffineTransform::new();
        node.borrow_mut().node_to_world(&mut aft, gdata);
        aft.invert();
        let mut inv = context.get_view_space().inverse();
        inv.multiply(&aft);

        let mut node = Point::new();
        AffineTransform::transform_to_point(&device, &mut node, &inv);
        (node.x, node.y)
    }

    // ------------------------------------------------------------
    // Nodes
    // ------------------------------------------------------------
    pub fn node_id(node: &RNode) -> usize {
        let id: usize;
        let part = node.borrow();
        id = part.data().borrow().node.id();
        id
    }

    pub fn get_rnode_data(id: usize, gdata: &GlobalData) -> RNodeData {
        let gpart = gdata.find_node(&id);
        let upart = gpart.unwrap().borrow();
        upart.data().clone()
    }

    pub fn id_equal_node(id: usize, node: &RNode) -> bool {
        let n = node.borrow();
        if id == n.data().borrow().node.id() {
            return true;
        }

        false
    }

    pub fn find_node(id: usize, node: &RNode) -> RNode {
        if let Some(children) = node.borrow().get_children() {
            let ng = NodeFunctions::sub_find_node(id, children);
            if !ng.borrow().is_nil() {
                return ng.clone();
            }
        } else {
            if NodeFunctions::id_equal_node(id, node) {
                return node.clone();
            }
        }

        NodeNil::new()
    }

    fn sub_find_node(id: usize, children: &RefCell<Vec<RNode>>) -> RNode {
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                let ng = NodeFunctions::sub_find_node(id, sub_children);
                if !ng.borrow().is_nil() {
                    return ng.clone();
                }
            } else {
                if NodeFunctions::id_equal_node(id, child) {
                    return child.clone();
                }
            }
        }

        NodeNil::new()
    }

    // ------------------------------------------------------------
    // Timing
    // ------------------------------------------------------------
    /// Iterate through tree registering all node marked as registrable.
    pub fn register_timing_targets(node: &RNode, man: &mut NodeManager) {
        let no = node.borrow();
        if no.data().borrow().node.canbe_timing_target() {
            man.register_timing_target(node.clone());
        }

        if let Some(children) = node.borrow().get_children() {
            NodeFunctions::sub_register_timing_targets(children, man);
        }
    }

    fn sub_register_timing_targets(children: &RefCell<Vec<RNode>>, man: &mut NodeManager) {
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                let chi = child.borrow();
                if chi.data().borrow().node.canbe_timing_target() {
                    man.register_timing_target(child.clone());
                }
                NodeFunctions::sub_register_timing_targets(sub_children, man);
            } else {
                let chi = child.borrow();
                if chi.data().borrow().node.canbe_timing_target() {
                    man.register_timing_target(child.clone());
                }
            }
        }
    }

    pub fn unregister_timing_targets_by_id(node_id: usize, man: &mut NodeManager) {
        // Use scheduler to unregister based on id.
        man.unschedule_timing_target_by_id(node_id);
    }

    pub fn unregister_timing_targets(node: &RNode, man: &mut NodeManager) {
        let no = node.borrow();
        if no.data().borrow().node.canbe_timing_target() {
            man.unschedule_timing_target(node.clone());
        }

        if let Some(children) = node.borrow().get_children() {
            NodeFunctions::sub_unregister_timing_targets(children, man);
        }
    }

    fn sub_unregister_timing_targets(children: &RefCell<Vec<RNode>>, man: &mut NodeManager) {
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                let chi = child.borrow();
                if chi.data().borrow().node.canbe_timing_target() {
                    man.unschedule_timing_target(child.clone());
                }
                NodeFunctions::sub_unregister_timing_targets(sub_children, man);
            } else {
                let chi = child.borrow();
                if chi.data().borrow().node.canbe_timing_target() {
                    man.unschedule_timing_target(child.clone());
                }
            }
        }
    }

    // ------------------------------------------------------------
    // Debug
    // ------------------------------------------------------------
    pub fn print_tree(tree: &RNode) {
        println!("---------- Tree ---------------");
        let tre = tree.borrow();
        NodeFunctions::print_branch(0, tre.name(), tre.data().borrow().node.id());
        if let Some(children) = tre.get_children() {
            NodeFunctions::print_sub_tree(children, 1);
        }
        println!("-------------------------------");
    }

    fn print_sub_tree(children: &RefCell<Vec<RNode>>, level: usize) {
        for child in children.borrow().iter() {
            if let Some(sub_children) = child.borrow().get_children() {
                let chi = child.borrow();
                NodeFunctions::print_branch(level, chi.name(), chi.data().borrow().node.id());
                NodeFunctions::print_sub_tree(sub_children, level + 1);
            } else {
                let chi = child.borrow();
                NodeFunctions::print_branch(level, chi.name(), chi.data().borrow().node.id());
            }
        }
    }

    fn print_branch(level: usize, name: String, id: usize) {
        for _ in 0..level {
            print!("  ");
        }
        println!("{} ({})", name, id);
    }

    // pub fn register_for_io_events(data: &mut GlobalData, parent: RNode, child: usize) {
    //     let fin = NodeFunctions::find_node(child, &parent);
    //     if !fin.borrow().is_nil() {
    //         println!("Register '{}' for io events", fin.borrow().name());
    //         data.register_io_event_targets(fin.clone());
    //     } else {
    //         println!("Could not find ({}) to register", child);
    //     }

    //     // self.io_event_targets.push(node);
    //     println!("len: {}", data.io_event_targets_count());
    // }

    // pub fn unregister_for_io_events(data: &GlobalData, parent: RNode, child: usize) {
    //     // let contains = self.io_event_targets.contains(&node);
    //     // if contains {
    //     //     return;
    //     // }
    //     let fin = NodeFunctions::find_node(child, &parent);
    //     if !fin.borrow().is_nil() {
    //         println!("Register '{}' for io events", fin.borrow().name());
    //         let nb = fin.borrow();
    //         data.io_event_targets
    //             .retain(|n| n.borrow().id() != nb.data().borrow().node.id());
    //     } else {
    //         println!("Could not find ({}) to unregister", child);
    //     }

    //     // Keep all but the referenced `node`
    //     println!("un len: {}", data.io_event_targets.len());
    // }

    // --------------------------------------------------------------------------
    // Debug stuff (typically drawn in device space)
    // --------------------------------------------------------------------------
    pub fn render_stats(
        fps: f64,
        ups: f64,
        avg_ren_time: f64,
        avg_up_time: f64,
        avg_blit_time: f64,
        context: &mut Context,
        data: &GlobalData,
    ) {
        // Draws to device space (aka window space)
        context.set_draw_color(&Palette::WHITE(200));
        context.text(
            5,
            (data.window_height - 24) as i32,
            &format!(
                "Fps:{}, Ups:{:5.1}, ren:{:3.2}, upd: {:3.2} Blt:{:5.2}ms",
                fps, ups, avg_ren_time, avg_up_time, avg_blit_time
            ),
            2,
            1,
        );
    }

    pub fn render_coordinates(context: &mut Context, data: &GlobalData) {
        // Draws to device space (aka window space)
        context.set_draw_color(&Palette::LIME());
        context.text(
            10,
            10,
            &format!("M: {}, {}", data.mouse.0, data.mouse.1),
            2,
            1,
        );

        context.text(
            10,
            30,
            &format!("V: {:5.2}, {:5.2}", data.view.0, data.view.1),
            2,
            1,
        );
    }
}
