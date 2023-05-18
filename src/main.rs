use raycast::{mainloop, palette::PALETTE, prelude::*, wl6};
use sdl2::{
    audio::AudioSpecDesired,
    event::Event,
    keyboard::Scancode,
    mixer::{InitFlag, AUDIO_U8, DEFAULT_CHANNELS},
    mouse::MouseButton,
    pixels::PixelFormatEnum,
    EventPump,
};

fn input_state_from_sdl_events(events: &mut EventPump) -> InputState {
    let mut input_state = InputState::default();

    for event in events.poll_iter() {
        match event {
            Event::Quit { .. } => input_state.quit = true,
            Event::KeyDown {
                scancode: Some(scancode),
                repeat: false,
                ..
            } => match scancode {
                Scancode::Escape => input_state.quit = true,
                Scancode::F1 => input_state.restart = true,
                Scancode::F2 => input_state.prev_level = true,
                Scancode::F3 => input_state.next_level = true,
                Scancode::F5 => input_state.save = true,
                Scancode::F6 => input_state.load = true,
                Scancode::F7 => input_state.toggle_stop_the_world = true,
                Scancode::F9 => input_state.toggle_render_alternative = true, // can be used e.g. to toggle between different draw impls at runtime
                Scancode::Tab => input_state.toggle_automap = true,
                Scancode::Grave => input_state.toggle_mouse_grab = true,
                Scancode::Num1 => input_state.select_weapon = Some(1),
                Scancode::Num2 => input_state.select_weapon = Some(2),
                Scancode::Num3 => input_state.select_weapon = Some(3),
                Scancode::Num4 => input_state.select_weapon = Some(4),
                Scancode::LeftBracket => input_state.misc_selection -= 1,
                Scancode::RightBracket => input_state.misc_selection += 1,

                _ => (),
            },
            Event::MouseMotion { xrel, yrel, .. } => {
                input_state.dx += xrel;
                input_state.dy += yrel;
            }
            _ => (),
        }
    }
    let keyboard_state = events.keyboard_state();
    if keyboard_state.is_scancode_pressed(Scancode::LShift) {
        input_state.misc_selection *= 10;
    };
    input_state.forward = keyboard_state.is_scancode_pressed(Scancode::W);
    input_state.backward = keyboard_state.is_scancode_pressed(Scancode::S);
    const HOLD_STRAFE: bool = true;
    if !HOLD_STRAFE {
        input_state.turn_left = keyboard_state.is_scancode_pressed(Scancode::A);
        input_state.turn_right = keyboard_state.is_scancode_pressed(Scancode::D);
        input_state.strafe_left = keyboard_state.is_scancode_pressed(Scancode::Q);
        input_state.strafe_right = keyboard_state.is_scancode_pressed(Scancode::E);
    } else {
        input_state.turn_left = keyboard_state.is_scancode_pressed(Scancode::Q);
        input_state.turn_right = keyboard_state.is_scancode_pressed(Scancode::E);
        input_state.strafe_left = keyboard_state.is_scancode_pressed(Scancode::A);
        input_state.strafe_right = keyboard_state.is_scancode_pressed(Scancode::D);
    }
    input_state.slow = keyboard_state.is_scancode_pressed(Scancode::LShift);
    input_state.open = keyboard_state.is_scancode_pressed(Scancode::Space);
    input_state.shoot = keyboard_state.is_scancode_pressed(Scancode::LCtrl);
    input_state.fast_forward_mode = keyboard_state.is_scancode_pressed(Scancode::F8);
    let mouse_state = events.mouse_state();
    input_state.shoot |= mouse_state.is_mouse_button_pressed(MouseButton::Left);
    input_state
}
struct SdlSoundChunks {
    chunks: Vec<sdl2::mixer::Chunk>,
}
impl SdlSoundChunks {
    pub fn new(resources: &Resources) -> Self {
        let chunks = resources
            .digisounds
            .sounds
            .iter()
            .map(|buf| sdl2::mixer::Chunk::from_raw_buffer(buf.clone().into_boxed_slice()).unwrap())
            .collect();
        Self { chunks }
    }
}
impl mainloop::AudioService for SdlSoundChunks {
    fn play_sound(&self, id: i32) {
        sdl2::mixer::Channel::all()
            .play(&self.chunks[id as usize], 0)
            .unwrap();
    }
}
fn main() -> raycast::prelude::Result<()> {
    let mut buffer: Vec<u8> = vec![0; WIDTH * HEIGHT];

    let resources = Resources::load_wl6("vswap.wl6");
    let mut maps = wl6::MapsFile::open("maphead.wl6", "gamemaps.wl6");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _audio = sdl_context.audio().unwrap();
    let freq = 8000;
    let format = AUDIO_U8;
    let channels = 1;
    let chunk_size = 256;
    sdl2::mixer::open_audio(freq, format, channels, chunk_size).unwrap();
    let _mixer_context = sdl2::mixer::init(InitFlag::empty());
    sdl2::mixer::allocate_channels(4);
    let sound_chunks = SdlSoundChunks::new(&resources);
    let window = video_subsystem
        .window("Raycastle3D", WIDTH as u32 * 4, HEIGHT as u32 * 4)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_streaming(PixelFormatEnum::RGB24, 320, 200)?;

    let mut events = sdl_context
        .event_pump()
        .unwrap_or_else(|_| panic!("faild to get event pump"));

    let mut mainloop = Mainloop::spawn(SpawnInfo::StartLevel(0, None), &mut maps);
    let mut mouse_grabbed = false;
    let mut initial_ungrabbed = true;
    let mut last_misc_selection = 0;
    loop {
        let mut input_state = input_state_from_sdl_events(&mut events);
        input_state.misc_selection += last_misc_selection;
        last_misc_selection = input_state.misc_selection;
        mainloop.use_mouse_move = mouse_grabbed;
        mainloop.run(&input_state, &mut buffer, &resources, &sound_chunks);
        if input_state.quit {
            break;
        }
        if input_state.is_deconstruct() {
            mainloop = Mainloop::spawn(mainloop.deconstruct(&input_state), &mut maps);
        }

        if input_state.toggle_mouse_grab || (input_state.shoot && initial_ungrabbed) {
            mouse_grabbed = !mouse_grabbed;
            canvas.window_mut().set_grab(mouse_grabbed);
            sdl_context.mouse().set_relative_mouse_mode(mouse_grabbed);
            initial_ungrabbed = false;
        }

        texture
            .with_lock(None, |tex_buffer: &mut [u8], pitch: usize| {
                for y in 0..200 {
                    for x in 0..320 {
                        let offset = y * pitch + x * 3;
                        let s_offset = y * 320 + x;
                        let c32 = PALETTE[buffer[s_offset] as usize];
                        tex_buffer[offset] = (c32 >> 16) as u8;
                        tex_buffer[offset + 1] = (c32 >> 8) as u8;
                        tex_buffer[offset + 2] = c32 as u8;
                    }
                }
            })
            .unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }

    Ok(())
}
