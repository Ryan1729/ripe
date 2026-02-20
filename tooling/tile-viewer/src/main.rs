
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
    use TileIndex::{Wall, Floor};
    //
    // Update
    //
    if state.input.pressed_this_frame(Button::A) {
        state.tile_index = match state.tile_index {
            TileIndex::Wall(..) => TileIndex::Floor,
            TileIndex::Floor => TileIndex::Wall(0),
        };
    } else if state.input.pressed_this_frame(Button::UP) {
        if let Wall(mask) = &mut state.tile_index { 
            *mask = mask.wrapping_add(1);
        }
    } else if state.input.pressed_this_frame(Button::DOWN) {
        if let Wall(mask) = &mut state.tile_index { 
            *mask = mask.wrapping_sub(1);
        }
    } else if state.input.pressed_this_frame(Button::RIGHT) {
        if let Wall(mask) = &mut state.tile_index { 
            *mask = mask.wrapping_add(16);
        }
    } else if state.input.pressed_this_frame(Button::LEFT) {
        if let Wall(mask) = &mut state.tile_index { 
            *mask = mask.wrapping_sub(16);
        }
    }


    //
    // Render
    //
    state.commands.begin_frame(&mut 0);

    let commands = &mut state.commands;

    let draw_tile_index = |commands: &mut Commands, xy, tile_index| {
        let label = format!("{tile_index:?}").to_lowercase();
        commands.print_line(label.as_bytes(), xy, 6);
    };

    draw_tile_index(commands, <_>::default(), state.tile_index);

    let wall_spec = &state.specs.wall;
    let floor_spec = &state.specs.floor;

    let draw_tile_sprite = |commands: &mut Commands, xy: unscaled::XY, tile_index: TileIndex| {
        let (rect, s_xy) = match tile_index {
            TileIndex::Wall(index) => (
                wall_spec.rect(xy),
                wall_spec.xy_from_tile_sprite(index),
            ),
            TileIndex::Floor => (
                floor_spec.rect(xy),
                floor_spec.xy_from_tile_sprite(0u16),
            ),
        };

        commands.sspr(
            s_xy,
            command::Rect::from_unscaled(rect),
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

    // TODO this seems wrong in some cases. probably worth pulling out into a function and writing a few unit tests
    let tile_indexes = neighboring_demo_indexes(state.tile_index);

    assert_eq!(xys.len(), tile_indexes.len());

    for i in 0..xys.len() {
        draw_tile_sprite(commands, xys[i], tile_indexes[i]);
    }
    
    {
        use unscaled::{XY, X, Y};
        let (x1, x2, x3) = (X(200), X(300), X(400));
        let (y1, y2, y3) = (Y(100), Y(120), Y(140));

        draw_tile_index(commands, XY { x: x1, y: y1 }, tile_indexes[0]);
        draw_tile_index(commands, XY { x: x2, y: y1 }, tile_indexes[1]);
        draw_tile_index(commands, XY { x: x3, y: y1 }, tile_indexes[2]);

        draw_tile_index(commands, XY { x: x1, y: y2 }, tile_indexes[3]);
        draw_tile_index(commands, XY { x: x2, y: y2 }, tile_indexes[4]);
        draw_tile_index(commands, XY { x: x3, y: y2 }, tile_indexes[5]);

        draw_tile_index(commands, XY { x: x1, y: y3 }, tile_indexes[6]);
        draw_tile_index(commands, XY { x: x2, y: y3 }, tile_indexes[7]);
        draw_tile_index(commands, XY { x: x3, y: y3 }, tile_indexes[8]);
    }

    state.commands.end_frame();

    state.input.previous_gamepad = state.input.gamepad;

    (state.commands.slice(), state.spritesheet.slice())
}

fn neighboring_demo_indexes(tile_index: TileIndex) -> [TileIndex; 9] {
    use TileIndex::{Wall, Floor};
    use sword::{LOWER_RIGHT, LOWER_MIDDLE, LOWER_LEFT, RIGHT_MIDDLE, LEFT_MIDDLE, UPPER_RIGHT, UPPER_MIDDLE, UPPER_LEFT};

    let neighbor_mask = match tile_index {
        Wall(mask) => mask,
        Floor => return [Floor; 9],
    };

    let neighboring_demo_index = |
        // "me" refers to the to-be-constructed index, and "base" refers to `state.tile_index`
        (from_base_to_me_mask, from_me_to_base_mask): (NeighborFlag, NeighborFlag),
        adjacent_assignment_masks: &[(NeighborFlag, NeighborFlag)],
    | {
        let variant_fn = if (neighbor_mask & from_base_to_me_mask.get()) != 0 {
            return Floor
        } else {
            Wall
        };

        let mut output_mask = 0;

        for (base_mask, me_mask) in adjacent_assignment_masks {
            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
            // we can use highest_one instead
            let base_shift = NeighborMask::BITS - 1 - base_mask.leading_zeros();
            let me_shift = NeighborMask::BITS - 1 - me_mask.leading_zeros();

            output_mask |= ((neighbor_mask & base_mask.get()) >> base_shift) << me_shift;
        }

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
            &[(UPPER_LEFT, UPPER_MIDDLE), (UPPER_MIDDLE, UPPER_RIGHT), (LOWER_LEFT, LOWER_MIDDLE), (LOWER_MIDDLE, LOWER_RIGHT)]
        ),
        tile_index,
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
}

#[cfg(test)]
mod neighboring_demo_indexes_works_on {
    use super::*;
    use TileIndex::{Wall, Floor};
    #[allow(unused_imports)]
    use sword::{LOWER_RIGHT, LOWER_MIDDLE, LOWER_LEFT, RIGHT_MIDDLE, LEFT_MIDDLE, UPPER_RIGHT, UPPER_MIDDLE, UPPER_LEFT};

    #[test]
    fn wall_0() {
        let w0 = neighboring_demo_indexes(Wall(0b0000_0000));

        assert_eq!(w0, [Wall(0); 9]);
    }

    #[test]
    fn wall_1() {
        let w1 = neighboring_demo_indexes(Wall(0b0000_0001));

        let expected = [
            Floor,
            Wall(LEFT_MIDDLE.get()),
            Wall(0),
            Wall(UPPER_MIDDLE.get()),
            Wall(0b0000_0001),
            Wall(0),
            Wall(0),
            Wall(0),
            Wall(0),
        ];

        assert_eq!(w1, expected);
    }

    #[test]
    fn wall_254() {
        let w254 = neighboring_demo_indexes(Wall(0b1111_1110));

        assert_ne!(
            w254[0],
            Wall(0b0000_0001)
        );
        assert!(matches!(w254[0], Wall(_)));
        if let Wall(neighbor_mask) = w254[0] {
            assert!((neighbor_mask & RIGHT_MIDDLE.get()) != 0);
            assert!((neighbor_mask & LOWER_MIDDLE.get()) != 0);
        }
    }

    #[test]
    fn wall_255() {
        let w255 = neighboring_demo_indexes(Wall(0b1111_1111));

        let expected = [
            Floor,
            Floor,
            Floor,
            Floor,
            Wall(0b1111_1111),
            Floor,
            Floor,
            Floor,
            Floor,
        ];

        assert_eq!(w255, expected);
    }
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