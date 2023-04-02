use std::cell::RefCell;
use std::collections::HashSet;
use std::time::{Duration, Instant};

use raycast::{palette::PALETTE, prelude::*, wl6};
use softbuffer::GraphicsContext;
use winit::event::{Event, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    let resources = Resources::load_wl6("vswap.wl6");
    let mut maps = wl6::MapsFile::open("maphead.wl6", "gamemaps.wl6");

    let mut mainloop = Some(Mainloop::spawn(SpawnInfo::StartLevel(0, None), &mut maps));
    let mouse_grabbed = false;
    let mut initial_ungrabbed = true;
    let mut buffer = vec![0u8; WIDTH * HEIGHT];
    // let mut rgb_buffer = vec![0u32; WIDTH * HEIGHT];
    let mut pressed_keys = HashSet::<VirtualKeyCode>::new();
    event_loop.run(move |event, _, control_flow| {
        // *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_micros(16667));
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let rgb_buffer = buffer.iter().map(|p| PALETTE[*p as usize]).collect::<Vec<_>>();
                graphics_context.set_buffer(&rgb_buffer, WIDTH as u16, HEIGHT as u16);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    },
                window_id,
            } if window_id == window.id() => {
                if let Some(vk) = input.virtual_keycode {
                    match input.state {
                        winit::event::ElementState::Pressed => {
                            pressed_keys.insert(vk);
                        }
                        winit::event::ElementState::Released => {
                            pressed_keys.remove(&vk);
                        }
                    }
                }
            }
            Event::MainEventsCleared => {
                let input_state = InputState {
                    forward: pressed_keys.contains(&VirtualKeyCode::W),
                    strafe_left: pressed_keys.contains(&VirtualKeyCode::A),
                    backward: pressed_keys.contains(&VirtualKeyCode::S),
                    strafe_right: pressed_keys.contains(&VirtualKeyCode::D),
                    ..Default::default()
                };

                if let Some(mainloop) = &mut mainloop {
                    println!("{input_state:?}");
                    let mut mainloop = mainloop;
                    mainloop.use_mouse_move = mouse_grabbed;
                    mainloop.run(&input_state, &mut buffer, &resources);
                    if input_state.quit {
                        // break;
                        todo!();
                    }
                }
                if input_state.is_deconstruct() {
                    let old_mainloop = mainloop.take().expect("mainloop None"); // mainloop is re-initialized immediately in the next line (assuming caught panics are not an issue here)
                    mainloop = Some(Mainloop::spawn(old_mainloop.deconstruct(&input_state), &mut maps));
                }

                // if input_state.toggle_mouse_grab || (input_state.shoot && initial_ungrabbed) {
                //     mouse_grabbed = !mouse_grabbed;
                //     canvas.window_mut().set_grab(mouse_grabbed);
                //     sdl_context.mouse().set_relative_mouse_mode(mouse_grabbed);
                //     initial_ungrabbed = false;
                // }

                window.request_redraw();
            }
            _ => {}
        }
    });
}
