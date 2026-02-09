
use gfx::{Commands};
use models::{Spritesheet};

struct State {
    commands: Commands,
    spritesheet: Spritesheet,
}

fn frame(state: &mut State) -> (&[platform_types::Command], (&[gfx_sizes::ARGB], usize)) {
    (state.commands.slice(), state.spritesheet.slice())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    args.next(); // exe name

    let pak = pak::from_reader(
        std::fs::File::open(
            args.next()
            .ok_or("A .pak filename is required!")?
        )?
    )?;

    run(State{
        commands: Commands::new(new_seed(), pak.specs.base_font.clone(), pak.specs.base_ui.clone()),
        spritesheet: pak.spritesheet,
    });

    Ok(())
}

fn new_seed() -> xs::Seed {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let time = time.as_secs_f64();

    unsafe {
        core::mem::transmute::<[f64; 2], [u8; 16]>([time, 1.0 / time])
    }
}

fn run(mut state: State) {
    use softbuffer::GraphicsContext;

    use winit::{
        event::{Event, WindowEvent},
        event_loop::{EventLoop, ControlFlow},
        window::WindowBuilder,
    };
    
    use render::{clip, FrameBuffer, NeedsRedraw};

    let event_loop = EventLoop::new();

    let builder = WindowBuilder::new()
        .with_title("tile viewer");

    let window = builder
        .build(&event_loop)
        .unwrap();

    let mut output_frame_buffer = {
        let size = window.inner_size();

        FrameBuffer::from_size((size.width as clip::W, size.height as clip::H))
    };

    let mut graphics_context = unsafe { GraphicsContext::new(window) }.unwrap();

    let mut loop_helper = spin_sleep::LoopHelper::builder()
            .build_with_target_rate(60.0);

    let mut just_gained_focus = true;

    event_loop.run(move |event, _, control_flow| {
        let window = graphics_context.window();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput{
                    input: winit::event::KeyboardInput {
                        state: element_state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    ..
                },
                window_id,
            } if window_id == window.id() => {
                use winit::event::{ElementState, VirtualKeyCode as VK};

                dbg!(keycode);
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(true),
                window_id,
            } if window_id == window.id() => {
                just_gained_focus = true;
            }
            Event::MainEventsCleared => {
                let (commands, gfx) = frame(&mut state);

                {
                    let size = window.inner_size();
                    output_frame_buffer.width = size.width as u16;
                    output_frame_buffer.height = size.height as u16;
                }

                let needs_redraw = render::render(
                    &mut output_frame_buffer,
                    commands,
                    gfx,
                );

                if NeedsRedraw::Yes == needs_redraw
                || just_gained_focus {
                    graphics_context.set_buffer(
                        &output_frame_buffer.buffer,
                        output_frame_buffer.width,
                        output_frame_buffer.height,
                    );
                }

                just_gained_focus = false;

                loop_helper.loop_sleep();
                loop_helper.loop_start();
            }
            _ => (),
        }
    });
}
