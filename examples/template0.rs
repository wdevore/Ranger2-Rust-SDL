// Run this example from the Ranger2-Rust-SDL directory:
// $ cargo run --example template0

extern crate ranger;

use ranger::{
    nodes::{node_functions::NodeFunctions, scenes::scene_boot::SceneBoot},
    world::World,
};

mod template_0;

use template_0::game_scene::GameScene;
use template_0::splash_scene::SplashScene;

const DISPLAY_RATIO: f32 = 16.0 / 9.0;
const WIDTH: u32 = 1024 + 512;
// Larget number causes the view to encompass more of the world
// which makes objects appear smaller.
const VIEW_SCALE: f64 = 1.5;

fn main() {
    // Use the Ranger engine to configure, boot and launch game.
    let window_width = WIDTH;
    let window_height = (WIDTH as f32 / DISPLAY_RATIO) as u32;

    let view_width = window_width as f64 * VIEW_SCALE;
    let view_height = window_height as f64 * VIEW_SCALE;

    println!("Display dimensions: [{} x {}]", window_width, window_height);
    println!("View dimensions: [{} x {}]", view_width, view_height);

    let mut world = match World::new(
        window_width,
        window_height,
        view_width,
        view_height,
        true,
        "Ranger2 Basic",
        "game.config",
        true,
    ) {
        Ok(eng) => eng,
        Err(err) => {
            panic!("Could not create Engine: {}", err);
        }
    };

    match world.configure() {
        Ok(msg) => {
            if msg != "Configured" {
                panic!("Unknown Configured response: {}", msg);
            }
        }
        Err(err) => {
            panic!("Error during Configured sequence: {}", err);
        }
    }

    match world.launch(build) {
        Ok(msg) => {
            println!("World: {}", msg);
        }
        Err(err) => {
            panic!("Error during launch and/or exit sequence: {}", err);
        }
    }
}

fn build(world: &mut World) -> bool {
    println!("Building...");
    let game_scene = GameScene::new("GameScene", world);

    NodeFunctions::register_timing_targets(&game_scene, world.node_manager_mut());

    let game_id = NodeFunctions::node_id(&game_scene);

    let splash_scene = SplashScene::with_replacement("SplashScene", game_id, world);
    {
        let mut splash = splash_scene.borrow_mut();

        if let Some(n) = splash.as_any_mut().downcast_mut::<SplashScene>() {
            n.pause_for_seconds(0.25);
        }
    }
    let splash_id = NodeFunctions::node_id(&splash_scene);

    NodeFunctions::register_timing_targets(&splash_scene, world.node_manager_mut());
    {
        let splash = splash_scene.borrow();
        println!("{} ({})", splash.name(), splash_id);
    }

    let boot_scene = SceneBoot::with_replacement("BootScene", splash_id, world);
    let boot_id = NodeFunctions::node_id(&boot_scene);
    {
        let boot = boot_scene.borrow();
        println!("{} ({})", boot.name(), boot_id);
    }

    world.push_node(boot_scene);

    println!(
        "Build complete. Node count: ({})",
        world.data().node_count()
    );
    world.data().print_pool();

    true
}
