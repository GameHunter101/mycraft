use std::{cell::RefCell, rc::Rc};

use chunk::{Chunk, X_SIZE, Y_SIZE, Z_SIZE};
use gamezap::{camera::CameraManager, module_manager::ModuleManager, texture::Texture, GameZap};
use nalgebra as na;
use sdl2::{event::WindowEvent, keyboard::Scancode, mouse::RelativeMouseState};
use utils::MeshTools;

mod chunk;
mod cube;
mod utils;

const ATLAS_SIZE: f32 = 256.0;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let window = Rc::new(video_subsystem.window("MyCraft", 800, 600).build().unwrap());
    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.15,
        b: 0.6,
        a: 1.0,
    };

    let module_manager = ModuleManager::builder()
        .mesh_manager()
        .camera_manager(na::Vector3::new(0.0, 0.0, 0.0), 0.0, 0.0, 45.0, 0.005)
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
        .build();

    let renderer = RefCell::new(engine.renderer);
    let renderer_device = &renderer.borrow().device;
    let renderer_queue = &renderer.borrow().queue;

    let renderer_borrow = renderer.borrow();

    let mut material_manager = renderer_borrow.module_manager.material_manager.borrow_mut();

    let texture_atlas = material_manager.new_material(
        "Texture Atlas",
        renderer_device,
        Some(
            pollster::block_on(Texture::load_texture(
                "atlas.png",
                renderer_device,
                renderer_queue,
                false,
            ))
            .unwrap(),
        ),
        None,
    );

    // let plain_material =
    //     material_manager.new_material("Plain Material", renderer_device, None, None);

    drop(material_manager);

    let mesh_manager = renderer_borrow
        .module_manager
        .mesh_manager
        .as_ref()
        .unwrap();

    let mut chunk_blocks = Box::new([[[1; Z_SIZE]; X_SIZE]; Y_SIZE]);
    chunk_blocks[Y_SIZE - 1] = [[0; Z_SIZE]; X_SIZE];

    let chunk = Chunk {
        position: na::Vector3::new(0.0, 0.0, 10.0),
        blocks: chunk_blocks,
        atlas_material_index: texture_atlas.1,
    };

    chunk.create_mesh(renderer_device, mesh_manager.borrow_mut());

    renderer_borrow.prep_renderer();

    'running: loop {
        for event in engine.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    break 'running;
                }
                sdl2::event::Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => renderer.borrow_mut().resize((width as u32, height as u32)),
                _ => {}
            }
        }
        let scancodes = engine
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .collect::<Vec<_>>();
        let mouse_state = engine.event_pump.relative_mouse_state();
        input(
            renderer_borrow.module_manager.camera_manager.as_ref(),
            &scancodes,
            &mouse_state,
        );
        renderer_borrow.update_buffers();
        renderer_borrow.render().unwrap();

        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 144));
    }
}

fn input(
    camera_manager: Option<&RefCell<CameraManager>>,
    scancodes: &Vec<Scancode>,
    mouse_state: &RelativeMouseState,
) {
    if let Some(camera_manager) = camera_manager {
        let camera_manager = camera_manager.borrow();
        let mut camera = camera_manager.camera.borrow_mut();
        camera.transform_camera(scancodes, mouse_state, true);
    }
}
