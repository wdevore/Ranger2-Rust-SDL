extern crate sdl2;

// use std::error::Error;
// use std::fmt;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use self::sdl2::{
    render::WindowCanvas,
    Sdl,
    {event::Event, keyboard::Keycode},
};

use events::io_events::IOEventData;
use nodes::{node_functions::NodeFunctions, node_manager::NodeManager, node_trait::RNode};
use rendering::render_context::Context;

// Game developer uses this callback to build their game.
type BuildCallback = fn(&mut World) -> bool;
pub type RCCanvas = Rc<RefCell<WindowCanvas>>;

pub struct GlobalData {
    pub window_width: usize,
    pub window_height: usize,
    pub view_width: f64,
    pub view_height: f64,
    pub view_centered: bool,
    pub title: String,
    pub config: String,
    pub vysnc_enabled: bool,
    pub perform_clear: bool,

    // Mouse-space is synonymous with window/device space.
    pub mouse: (i32, i32), // (x,y)
    pub mouse_changed: bool,

    // View-space coordinates
    pub view: (f64, f64),
    // node-space coordinates
    // node: (f64, f64),

    // Event targets
    io_event_targets: Vec<RNode>,

    // Collect all nodes in the system for fast access
    node_pool: HashMap<usize, RNode>,
}

impl Drop for GlobalData {
    fn drop(&mut self) {
        println!(
            "Dropping GlobalData: targets ({}), nodes; ({})",
            self.io_event_targets.len(),
            self.node_pool.len()
        );

        self.io_event_targets.clear();
        self.node_pool.clear();
    }
}

impl GlobalData {
    fn new() -> Self {
        Self {
            window_width: 0,
            window_height: 0,
            view_width: 0.0,
            view_height: 0.0,
            view_centered: true,
            title: String::from(""),
            config: String::from("config.json"),
            vysnc_enabled: true,
            perform_clear: true,

            mouse: (0, 0),
            view: (0.0, 0.0),
            // node: (0.0, 0.0),
            mouse_changed: false,

            io_event_targets: Vec::new(),

            node_pool: HashMap::new(),
        }
    }

    pub fn set_mouse(&mut self, x: i32, y: i32) {
        self.mouse = (x, y);
        self.mouse_changed = true;
    }

    pub fn update_view_coords(&mut self, context: &mut Context) {
        if self.mouse_changed {
            self.view = NodeFunctions::map_device_to_view(self.mouse.0, self.mouse.1, context);
            self.mouse_changed = false;
        }
    }

    pub fn add_node(&mut self, node: RNode) {
        let id: usize;
        {
            let no = node.borrow();
            id = no.data().borrow().node.id();
        }
        self.node_pool.insert(id, node);
    }

    pub fn find_node(&self, id: &usize) -> Option<&RNode> {
        self.node_pool.get(id)
    }

    pub fn take_node(&mut self, id: &usize) -> Option<RNode> {
        // println!("take_node is taking: ({}) from pool", id);
        self.node_pool.remove(id)
    }

    pub fn node_count(&self) -> usize {
        self.node_pool.len()
    }

    pub fn print_pool(&self) {
        println!("__~__~__~__~__~__~ NODE POOL __~__~__~__~__~__~__~__~");
        for (k, v) in self.node_pool.iter() {
            println!("{} ({})", v.borrow().name(), k);
        }
        println!("__~__~__~__~__~__~__~__~__~__~__~__~__~__~__~__~");
    }

    pub fn register_io_event_targets(&mut self, node: RNode) {
        self.io_event_targets.push(node);
    }

    pub fn io_event_targets_count(&self) -> usize {
        self.io_event_targets.len()
    }
}

const SECOND: u32 = 1000000000; // billion ns in a second

// If Sleep is disabled or removed from the code, or VSync is enabled
// then the below FPS variable has no effect.
const FRAMES_PER_SECOND: usize = 120; // Minimum fps to achieve

const UPDATES_PER_SECOND: usize = 30; // Maximum updates per second

// 1 frame period is equal to a fraction. For example, if
// FRAMES_PER_SECOND = 60.0 then frame period is 0.01666666667s of a second
// or in milliseconds it is 1000.0/60.0 = 16.66666667ms per frame.
// 1ms = 1000us = 1000000ns
const FRAME_PERIOD: f64 = 1_000_000_000.0 / FRAMES_PER_SECOND as f64; // in nanoseconds

const UPDATE_PERIOD: f64 = 1_000_000_000.0 / UPDATES_PER_SECOND as f64; // in nanoseconds

/// Ranger is the main object hosting your game. You construct [Scene]s and give them to Ranger
/// for execution. When the last Scene exits the game comes to an end.
// #[derive(Clone)]
pub struct World {
    data: GlobalData,

    node_manager: NodeManager,

    context: Sdl,
    config: String,

    id: usize,
}

impl Drop for World {
    fn drop(&mut self) {
        println!("Dropping: 'World'");
    }
}

impl World {
    /// Create a Ranger game `Engine`.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of gui window
    /// * `height` - Height of gui window
    /// * `title` - Window's title bar text
    /// * `config` - JSON configuration file. For example, contains the window background color
    pub fn new(
        window_width: u32,
        window_height: u32,
        view_width: f64,
        view_height: f64,
        view_centered: bool,
        title: &str,
        config: &str,
        vysnc_enabled: bool,
    ) -> Result<Self, String> {
        let mut data = GlobalData::new();
        data.window_width = window_width as usize;
        data.window_height = window_height as usize;
        data.view_width = view_width;
        data.view_height = view_height;
        data.view_centered = view_centered;
        data.title = String::from(title);

        // --------------------------------------------------------------
        // SDL configuration
        // --------------------------------------------------------------
        let sdl_context = match sdl2::init() {
            Ok(context) => context,
            Err(err) => return Err(err),
        };

        let video_subsystem = match sdl_context.video() {
            Ok(system) => system,
            Err(err) => return Err(err),
        };

        let window = match video_subsystem
            .window(title, window_width, window_height)
            .position_centered()
            .build()
        {
            Ok(win) => win,
            Err(build_error) => return Err(build_error.to_string()),
        };

        let canvas = if vysnc_enabled {
            match window.into_canvas().present_vsync().build() {
                Ok(can) => can,
                Err(err) => return Err(err.to_string()),
            }
        } else {
            match window.into_canvas().build() {
                Ok(can) => can,
                Err(err) => return Err(err.to_string()),
            }
        };

        let man = NodeManager::new(canvas, &data);

        let e = Self {
            data: data,
            context: sdl_context,
            node_manager: man,
            config: config.to_string(),
            id: 0,
        };

        Ok(e)
    }

    pub fn gen_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    /// Configure using config json
    pub fn configure(&mut self) -> Result<String, String> {
        println!("Using config: {}", self.config);

        // Err(String::from("kaboom"))
        Ok(String::from("Configured"))
    }

    pub fn launch(&mut self, build: BuildCallback) -> Result<String, String> {
        // Perform pre-build of underlying Systems (SceneManager, Scheduler, TweenManager...)
        println!("Constructing and/or initializing Systems...");

        // Now notify the developer to build their game.
        let built = build(self);
        if !built {
            return Err(String::from("Game failed to build."));
        }

        println!("Launching game...");
        self.core_loop()?;

        // Shutdown engine

        Ok(String::from("Exited"))
    }

    // ---------------------------------------------------------------
    // Properties
    // ---------------------------------------------------------------
    pub fn data(&self) -> &GlobalData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut GlobalData {
        &mut self.data
    }

    // ---------------------------------------------------------------
    // Node management
    // ---------------------------------------------------------------
    pub fn node_manager_mut(&mut self) -> &mut NodeManager {
        &mut self.node_manager
    }

    pub fn push_node(&self, scene: RNode) {
        self.node_manager.push_node(scene);
    }

    pub fn core_loop(&mut self) -> Result<String, String> {
        let update_period = Duration::new(0, UPDATE_PERIOD.round() as u32);
        let ns_per_update = update_period.subsec_nanos();
        let frame_dt = ns_per_update as f64 / 1000000.0;
        println!("ns_per_update: {}", ns_per_update);
        let frame_period = Duration::new(0, FRAME_PERIOD.round() as u32);

        let mut avg_blit_time = 0.0;
        let mut avg_ren_time = 0.0;
        let mut avg_up_time = 0.0;

        let mut lag = 0;
        let mut second_acm = 0;
        let mut fps = 0;
        let mut fps_cnt = 0;
        let mut ups = 0f64;
        let mut ups_cnt = 0;

        // #[allow(unused_assignments)]
        // let mut sleep = Duration::new(0, 0);
        // let mut sleep_accum = 0u32;
        let mut blit_accum = 0u32;
        let mut proc_accum = 0u32;
        let mut up_accum = 0u32;

        let mut previous_t = Instant::now();

        let mut keycode = Keycode::Clear;

        let mut event_pump = match self.context.event_pump() {
            Ok(pump) => pump,
            Err(err) => return Err(err),
        };

        'fast: loop {
            let current_t = Instant::now();

            // ##############################################################
            // Input
            // ##############################################################
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        .. // don't-care about other fields
                    } => break 'fast,
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        repeat: false,
                        ..
                    } => {
                        // Do something
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        keycode = Keycode::Right;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        keycode = Keycode::Left;
                    }
                    Event::KeyUp {
                        ..
                    } => {
                        keycode = Keycode::Clear;
                    }
                    Event::MouseMotion {
                        x,
                        y,
                        ..
                    } => {
                        // println!("mouse {},{}", x, y);
                        self.node_manager.io_event(IOEventData::new_mouse_event(x,y), &mut self.data);
                    }
                    _ => {}
                }
            }

            if keycode != Keycode::Clear {
                println!("key: {}", keycode);
            }

            // println!("================================================");

            // ##############################################################
            // Update
            // ##############################################################
            let elapsed_t = current_t - previous_t;
            previous_t = current_t;
            lag += elapsed_t.subsec_nanos();

            let u = Instant::now();

            'up: loop {
                if lag >= ns_per_update {
                    self.node_manager.update(frame_dt);
                    lag -= ns_per_update;
                    ups_cnt += 1;
                } else {
                    ups_cnt = 0;
                    break 'up;
                }
            }

            // ::std::thread::sleep(Duration::from_millis(15)); // force/test pipeline overload

            let un = Instant::now().duration_since(u);
            up_accum += un.subsec_nanos();

            // ##############################################################
            // Render
            // ##############################################################
            let interpolation = (lag as f64) / (ns_per_update as f64);
            let pn: Duration;
            {
                // let mut scm = scene_manager.borrow_mut();

                // If vsync is enabled then this takes nearly 1/fps milliseconds.
                // In other words it is waiting for the refresh vertical sync.
                self.node_manager.pre_visit();

                let p = Instant::now();

                if !self.node_manager.visit(interpolation, &mut self.data) {
                    // There are no more scenes to draw
                    break 'fast;
                }

                NodeFunctions::render_stats(
                    fps as f64,
                    ups as f64,
                    avg_ren_time,
                    avg_up_time,
                    avg_blit_time,
                    self.node_manager.context_mut(),
                    &self.data,
                );

                NodeFunctions::render_coordinates(self.node_manager.context_mut(), &self.data);

                pn = Instant::now().duration_since(p);
                // println!("render time: {}", pn.subsec_micros());
                proc_accum += pn.subsec_nanos();
            }

            // ##############################################################
            // Blit
            // ##############################################################
            // SDL appears to only take about 0.3ms to blit.
            let b = Instant::now();
            self.node_manager.post_visit();
            let bn = Instant::now().duration_since(b);

            // print!("[({}), {}]", blit_accum, bn.subsec_nanos());
            // print!("[{}]", bn.subsec_nanos() as f64 / 1000000.0);
            blit_accum += bn.subsec_nanos();

            // ##############################################################
            // Sleep
            // ##############################################################
            // How much time was taken for the above steps
            let work = un + bn + pn;

            // Was the work done in this frame less than the alotted period
            if work < frame_period {
                // Sleep is the remainder.
                // sleep = self.frame_period - work;
                // std::thread::sleep(sleep);

                // Simply sleep for a tiny amount to allow main thread processing.
                // std::thread::sleep(Duration::from_micros(500));
                // std::thread::sleep(Duration::from_millis(1));
                // std::thread::yield_now();
            } else {
                //sleep = Duration::new(0, 0);
            }

            // The total elapsed time spent for the frame is the work plus any
            // sleeping.
            // let elapsed = work; // + sleep;

            // let frame_time = elapsed.subsec_nanos();
            // Because we are counting using what amounts to nothing more
            // than the frame period then the second counter will be slightly off
            // by a frame period.
            second_acm += elapsed_t.subsec_nanos(); //frame_time;
            fps_cnt += 1;

            if second_acm >= SECOND {
                // println!("display update {}, {}", ns_per_update, SECOND);
                // (1 / (((ups+1) * ns_per_update) / 1000000)) * 1000 = fps
                fps = fps_cnt;
                ups = (1.0 / (((ups_cnt + 1) * ns_per_update) as f64 / 1000000.0)) * 1000.0;
                avg_blit_time = ((blit_accum as f64) / (fps as f64)) / 1000000.0;
                avg_ren_time = ((proc_accum as f64) / (fps as f64)) / 1000000.0;
                // self.avg_sleep_time = ((sleep_accum as f64) / (fps as f64)) / 1000000.0;
                avg_up_time = ((up_accum as f64) / (fps as f64)) / 1000000.0;

                // println!(
                //     "elap: {:8.5}, fps: {} ({:8.5}), up: {}, [slp: {:8.5}, ren: {:8.5}, aup: {:8.5}, blit: {:8.5}] = {:8.5}",
                //     frame_acm as f64 / 1000000.0,
                //     self.fps,
                //     1000.0 / total,
                //     self.ups,
                //     self.avg_sleep_time,
                //     self.avg_ren_time,
                //     self.avg_up_time,
                //     self.avg_blit_time,
                //     total ,
                // );

                second_acm = 0;
                // sleep_accum = 0;
                proc_accum = 0;
                blit_accum = 0;
                up_accum = 0;

                fps_cnt = 0;

                ups_cnt = 0;
            }

            // break 'fast; // Debug
        }

        Ok(String::from("Exited Game loop"))
    }
}
