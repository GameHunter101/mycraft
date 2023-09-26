use std::{
    borrow::BorrowMut,
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

use chunk_loader::ChunkLoader;
use gamezap::{
    module_manager::ModuleManager, renderer::Renderer, texture::Texture, EngineDetails,
    EngineSettings, EngineSystems, FrameDependancy, GameZap,
};
use nalgebra as na;
use sdl2::keyboard::Keycode;

mod chunk;
mod chunk_loader;
mod cube;
mod utils;

const ATLAS_SIZE: f32 = 256.0;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let window_size = (1200, 800);
    let window = Rc::new(
        video_subsystem
            .window("MyCraft", window_size.0, window_size.1)
            .resizable()
            .build()
            .unwrap(),
    );
    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.15,
        b: 0.6,
        a: 1.0,
    };

    let module_manager = ModuleManager::builder()
        .mesh_manager()
        .camera_manager(
            na::Vector3::new(0.0, 300.0, 0.0),
            0.1,
            7.0,
            0.0,
            0.0,
            45.0,
            0.01,
            1200.0,
            window_size.0 as f32,
            window_size.1 as f32,
        )
        .build();

    let mut engine = GameZap::builder()
        .window_and_renderer(
            sdl_context,
            video_subsystem,
            event_pump,
            window,
            clear_color,
        )
        .module_manager(module_manager)
        .antialiasing()
        .hide_cursor()
        .build();

    let renderer = &engine.renderer;
    let renderer_device = renderer.device.clone();
    let renderer_queue = renderer.queue.clone();

    let mut material_manager = renderer.module_manager.material_manager.borrow_mut();

    let texture_atlas = material_manager.new_material(
        "Texture Atlas",
        &renderer_device,
        Some(
            pollster::block_on(Texture::load_texture(
                "atlas.png",
                &renderer_device,
                &renderer_queue.clone(),
                false,
            ))
            .unwrap(),
        ),
        None,
    );

    drop(material_manager);

    let mesh_manager = renderer.module_manager.mesh_manager.as_ref().unwrap();

    let chunk_loader = ChunkLoader::load_chunks(
        na::Vector2::new(0.0, 0.0),
        texture_atlas.1,
        mesh_manager.lock().unwrap().diffuse_pipeline_models.len(),
        renderer_device.clone(),
        mesh_manager.clone(),
    );
    let chunk_loader_frame_dependancy: RefCell<Box<dyn FrameDependancy>> =
        RefCell::new(Box::new(chunk_loader));

    renderer.prep_renderer();

    drop(renderer);

    engine
        .keybinds
        .insert(Keycode::Escape, (Box::new(toggle_cursor), vec![]));

    engine.main_loop(vec![
        (Box::new(input), vec![]),
        (
            Box::new(recalculate_chunks),
            vec![chunk_loader_frame_dependancy.borrow_mut()],
        ),
    ]);
}

fn input(
    engine_details: RefMut<EngineDetails>,
    renderer: &Renderer,
    engine_systems: Ref<EngineSystems>,
    _frame_dependancies: &mut Vec<RefMut<Box<dyn FrameDependancy>>>,
) {
    let camera_manager = &renderer.module_manager.camera_manager;
    if let Some(camera_manager) = camera_manager {
        let camera_manager = camera_manager.borrow();
        let mut camera = camera_manager.camera.borrow_mut();
        if let Some(mouse_state) = engine_details.mouse_state.0 {
            camera.transform_camera(
                &engine_details.pressed_scancodes,
                &mouse_state,
                engine_systems
                    .sdl_context
                    .borrow()
                    .mouse()
                    .relative_mouse_mode(),
                engine_details.last_frame_duration.as_seconds_f32(),
            );
        }
    }
}

fn recalculate_chunks(
    engine_details: RefMut<EngineDetails>,
    renderer: &Renderer,
    engine_systems: Ref<EngineSystems>,
    frame_dependancies: &mut Vec<RefMut<Box<dyn FrameDependancy>>>,
) {
    let chunk_loader = frame_dependancies[0].borrow_mut();
    chunk_loader.frame_update(engine_details, renderer, engine_systems);
}

fn toggle_cursor(
    mut engine_details: RefMut<EngineDetails>,
    _renderer: &Renderer,
    engine_systems: Ref<EngineSystems>,
    _frame_dependancies: &mut Vec<RefMut<Box<dyn FrameDependancy>>>,
) {
    let old_mouse = engine_details.mouse_state.1;
    engine_details.mouse_state.1 = !old_mouse;
    engine_systems
        .sdl_context
        .borrow_mut()
        .update_cursor_mode(!old_mouse);
}
