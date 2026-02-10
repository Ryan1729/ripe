
///! A program to view tiles, to see how they visually join up next to other tiles.
///! Main purpose is to allow seeing each wall/floor tile next to each possible set of
///! neighboring tiles, to make sure we have all of those possibilities looking good.
// TODO? Make a Game of Life implementation using these tiles, just for fun?

use gfx::{Commands};
use models::{sprite, Spritesheet};
use platform_types::{command, unscaled, Button, Input};
use sword::{NeighborFlag, NeighborMask, TileIndex};


struct State {
    commands: Commands,
    input: Input,
    spritesheet: Spritesheet,
    specs: sprite::Specs,
    tile_index: TileIndex,
}

fn frame(state: &mut State) -> (&[platform_types::Command], (&[gfx_sizes::ARGB], usize)) {
    //
    // Update
    //
    if state.input.pressed_this_frame(Button::A) {
        state.tile_index = match state.tile_index {
            TileIndex::Wall(neighbors) => TileIndex::Floor(neighbors),
            TileIndex::Floor(neighbors) => TileIndex::Wall(neighbors),
        };
    } else if state.input.pressed_this_frame(Button::UP) {
        let mask = state.tile_index.neighbor_mask_mut();
        *mask = mask.wrapping_add(1);
    } else if state.input.pressed_this_frame(Button::DOWN) {
        let mask = state.tile_index.neighbor_mask_mut();
        *mask = mask.wrapping_sub(1);
    } else if state.input.pressed_this_frame(Button::RIGHT) {
        let mask = state.tile_index.neighbor_mask_mut();
        *mask = mask.wrapping_add(16);
    } else if state.input.pressed_this_frame(Button::LEFT) {
        let mask = state.tile_index.neighbor_mask_mut();
        *mask = mask.wrapping_sub(16);
    }


    //
    // Render
    //
    state.commands.begin_frame(&mut 0);

    let label = format!("{:?}", state.tile_index).to_lowercase();

    state.commands.print_line(label.as_bytes(), <_>::default(), 6);

    let mut draw_tile_sprite = |xy: unscaled::XY, tile_index: TileIndex| {
        state.commands.sspr(
            sprite::XY::default().apply(&state.specs.sword), // <- placeholder
            //todo!("Rearrange spritesheet to make this part simpler"),
            command::Rect::from_unscaled(state.specs.sword.rect(xy)),
        );
    };

    let tile = state.specs.sword.tile();

    let upper_left = unscaled::XY{ x: unscaled::X(100), y: unscaled::Y(100) };

    let xys = [
        upper_left,
        upper_left + tile.w,
        upper_left + tile.w + tile.w,
        upper_left + tile.h,
        upper_left + tile,
        upper_left + tile + tile.w,
        upper_left + tile.h + tile.h,
        upper_left + tile.h + tile,
        upper_left + tile + tile,
    ];

    let tile_indexes = {
        use TileIndex::{Wall, Floor};
        use sword::{LOWER_RIGHT, LOWER_MIDDLE, LOWER_LEFT, RIGHT_MIDDLE, LEFT_MIDDLE, UPPER_RIGHT, UPPER_MIDDLE, UPPER_LEFT};

        let neighbor_mask = state.tile_index.neighbor_mask();

        let is_floor_mask = match state.tile_index {
            Wall(..) => 0,
            Floor(..) => 1,
        };

        let neighboring_demo_index = |
            // "me" refers to the to-be-constructed index, and "base" refers to `state.tile_index`
            (from_base_to_me_mask, from_me_to_base_mask): (NeighborFlag, NeighborFlag),
            adjacent_assignment_masks: &[(NeighborFlag, NeighborFlag)],
        | {
            let variant_fn = if (neighbor_mask & from_base_to_me_mask.get()) != 0 {
                Floor
            } else {
                Wall
            };

            let mut output_mask = 0;

            for (base_mask, me_mask) in adjacent_assignment_masks {
                // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                // we can use highest_one instead
                let base_shift = NeighborMask::BITS - 1 - base_mask.leading_zeros();

                output_mask |= ((neighbor_mask & base_mask.get()) >> base_shift) << me_mask.get();
            }

            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
            // we can use highest_one instead
            let from_me_to_base_shift = NeighborMask::BITS - 1 - from_me_to_base_mask.leading_zeros();

            output_mask |= is_floor_mask << from_me_to_base_shift;

            variant_fn(output_mask)
        };

        [
            neighboring_demo_index(
                (UPPER_LEFT, LOWER_RIGHT),
                &[(UPPER_MIDDLE, RIGHT_MIDDLE), (LEFT_MIDDLE, LOWER_MIDDLE)]
            ),
            neighboring_demo_index(
                (UPPER_MIDDLE, LOWER_MIDDLE),
                &[(UPPER_LEFT, LEFT_MIDDLE), (LEFT_MIDDLE, LOWER_LEFT), (UPPER_RIGHT, RIGHT_MIDDLE), (RIGHT_MIDDLE, LOWER_RIGHT)],
            ),
            neighboring_demo_index(
                (UPPER_RIGHT, LOWER_LEFT),
                &[(UPPER_MIDDLE, LEFT_MIDDLE), (RIGHT_MIDDLE, LOWER_MIDDLE)]
            ),
            neighboring_demo_index(
                (LEFT_MIDDLE, RIGHT_MIDDLE),
                &[(UPPER_LEFT, UPPER_MIDDLE), (UPPER_MIDDLE, RIGHT_MIDDLE), (LOWER_LEFT, LOWER_MIDDLE), (LOWER_MIDDLE, LOWER_RIGHT)]
            ),
            state.tile_index,
            neighboring_demo_index(
                (RIGHT_MIDDLE, LEFT_MIDDLE),
                &[(UPPER_MIDDLE, UPPER_LEFT), (UPPER_RIGHT, UPPER_MIDDLE), (LOWER_MIDDLE, LOWER_LEFT), (LOWER_RIGHT, LOWER_MIDDLE)]
            ),
            neighboring_demo_index(
                (LOWER_LEFT, UPPER_RIGHT),
                &[(LEFT_MIDDLE, UPPER_MIDDLE), (LOWER_MIDDLE, RIGHT_MIDDLE)]
            ),
            neighboring_demo_index(
                (LOWER_MIDDLE, UPPER_MIDDLE),
                &[(LEFT_MIDDLE, UPPER_LEFT), (RIGHT_MIDDLE, UPPER_RIGHT), (LOWER_LEFT, LEFT_MIDDLE), (LOWER_RIGHT, RIGHT_MIDDLE)]
            ),
            neighboring_demo_index(
                (LOWER_RIGHT, UPPER_LEFT),
                &[(RIGHT_MIDDLE, UPPER_MIDDLE), (LOWER_MIDDLE, LEFT_MIDDLE)]
            ),
        ]
    };

    assert_eq!(xys.len(), tile_indexes.len());

    for i in 0..xys.len() {
        draw_tile_sprite(xys[i], tile_indexes[i]);
    }

    state.commands.end_frame();

    state.input.previous_gamepad = state.input.gamepad;

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
        input: <_>::default(),
        spritesheet: pak.spritesheet,
        specs: pak.specs,
        tile_index: TileIndex::default(),
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

                use platform_types::Button;

                let button = match keycode {
                    VK::Return => Button::START,
                    VK::RShift => Button::SELECT,
                    VK::Up => Button::UP,
                    VK::Left => Button::LEFT,
                    VK::Right => Button::RIGHT,
                    VK::Down => Button::DOWN,

                    VK::Z => Button::A,
                    VK::X => Button::B,

                    // For those using the Dvorak layout.
                    VK::Semicolon => Button::A,
                    VK::Q => Button::B,

                    _ => return,
                };

                match element_state {
                    ElementState::Pressed => press(&mut state, button),
                    ElementState::Released => release(&mut state, button),
                }
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

fn press(state: &mut State, button: Button) {
    if state.input.previous_gamepad.contains(button) {
        //This is meant to pass along the key repeat, if any.
        //Not sure if rewriting history is the best way to do this.
        state.input.previous_gamepad.remove(button);
    }

    state.input.gamepad.insert(button);
}

fn release(state: &mut State, button: Button) {
    state.input.gamepad.remove(button);
}