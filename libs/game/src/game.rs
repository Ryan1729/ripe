use models::{Entity, TileSprite, XY, xy};
use platform_types::{command, unscaled};
use xs::{Xs, Seed};

#[derive(Clone)]
pub struct Config {
    // TODO Nonempty Vec
    pub segments: Vec<WorldSegment>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            segments: vec![
                WorldSegment {
                    width: 1,
                    tiles: vec![
                        Tile {
                            sprite: 1,
                        }
                    ],
                }
            ],
        }
    }
}

#[derive(Clone, Default)]
pub struct Tile {
    pub sprite: TileSprite,
}

fn is_passable(tile: &Tile) -> bool {
    tile.sprite == models::FLOOR_SPRITE
}

pub type SegmentWidth = usize;

#[derive(Clone, Default)]
pub struct WorldSegment {
    pub width: SegmentWidth,
    // TODO? Nonempty Vec?
    // TODO Since usize is u32 on wasm, let's make a Vec32 type that makes that rsstriction clear, so we
    // can't have like PC only worlds that beak in weird ways online. Probably no one will ever need that
    // many tiles per segment. Plus, then xs conversions go away.
    pub tiles: Vec<Tile>,
}

fn random_passable_tile(rng: &mut Xs, segment: &WorldSegment) -> Option<XY> {
    // TODO? Cap tiles length or accept this giving a messed up probabilty for large segments?
    let len = segment.tiles.len();
    let offset = xs::range(rng, 0..len as u32) as usize;
    for index in 0..len {
        let i = (index + offset) % len;

        let tile = &segment.tiles[i];

        if is_passable(tile) {
            return Some(i_to_xy(segment, i));
        }
    }

    return None;
}

fn i_to_xy(segment: &WorldSegment, index: usize) -> XY {
    XY {
        x: xy::x((index % segment.width) as _),
        y: xy::y((index / segment.width) as _),
    }
}

#[derive(Clone, Default)]
pub struct World {
    // TODO a graph structure of `WorldSegment`s instead of just one
    pub segment: WorldSegment,
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub config: Config,
    pub world: World,
    pub player: Entity,
}

// Proposed Steps
// * Make the simplest task: Go find a thing and bring it to the person who wants it
//     * I think baking in things being parsed from a file early, makes sense
//         * Can start with an embedded string, just to exercise the parsing
//             * JSON I guess? OR is our own format better?
//                 * How painful is defining an ASCII map in JSON?
//     * Make the theme changeable, including graphics for now
//     * Will need to figure out how this works for the wasm version. Uploadable file?
//     * Will need to implement for desktop too, even if how it works is a little more clear
// * Fill out the other interactions:
//    * Get told that there is a specific thing by the one that wants it
//    * A proper "You won" screen
//        * Probably make this customizable too
// * Make it more complex by having a locked door that you get the key for by getting one person a thing, that prevents you from getting a second person a thing
// * Add a way to have just collecting a thing unlock a door
// * Add hallways between rooms that we'll figure out a way to make more interesting later
//    * Drain some resource, probably. Say HP that can be restored at the safe rooms
//        * So like a random halway with like one monster in it, for now
// * If not already randomized, randomize things like which tasks are done in which order, based on how they are locked behind each other
// * If not already, make more theme things changeable, and the script for charactrers, descriptions, etc.
// * Playtest a few rounds, see what feel like it needs expanding
//     * The hallways seem like a plausible example
//     * More variation in the safe rooms seems like another
// * Possible future ideas
//    * Sokoban blocks
//    * Ice sliding puzzles
//    * Arcade machines with minigames in them

// Substeps of "Go find a thing and bring it to the person who wants it"
// * Define an example of a text format, which can define at least one room's tiles for now, with room for expansion ✔
//    * Let's use TOML, mostly for comments.
//        * Hmm. https://stackoverflow.com/a/648298/ talks about just starting with a programming language as config. Maybe we should do that?
//            * On the other hand, we could just allow users to reference lua files from the TOML.
//              ... But that's reinventing part of lua requires, which we wouldn't have to do if
//              we just used lua from the start. Plus, if we eventually did do that, then there'd
//              still be lua requires in the system, making it more complicated
//                * Okay, why not just write the whole thing in lua then? Speed I guess? Not having to deal with a C impl?
//                    * Well that's an argument against a lua config at all!
//                        * Rhai then? It does support WASM! ... Apparently there's a bunch of pure rust scripting languages: https://www.boringcactus.com/2020/09/16/survey-of-rust-embeddable-scripting-languages.html
//                            * But only Rhai is mentioned as having WASM support on that page.
//                              And I wasn't able to find evidence that the nice looking ones had WASM support.
//                              So seems like Rhai is the winner, assuming we can do simple config well in it.
//    * Always include a version number!
//        * I think we can include that in the eventual pack file format? I guess we *could* make it a comment in the Rhai file?
//            * Currently a plain zip file with a manifest file that will have the version number seems like a reasonable design for the pack file
//                * Other stuff that could go in the manifest:
//                    * Author Name
//                    * (Optional) Description
//                    * (Optional) License?
//                    * (Optional) Specification of the icon to use for it? Player character by default
//    * Include a required way to define where the player starts, if they start in that room
//        * Not sure why I thought this should be required, other than yes validation will need to ensure that there is at least one floor tile. I imagine that most times starting at a random point will be fine, until it isn't
//            * Further, each tile definition should probably be a set of flags, for like is_passable, is_entrance, is_item_spawn_point, is_npc_spawn_point, etc.
// * Embed a string of that format in the program for now. ✔
// * Parse that string into the definition of the tiles ✔
//    * Leave room for a validate step after the parsing. Validation errors should eventually all contain custom error messages including why the given thing is needed.
//        * This can be done inside the parse function
// * Implement the player walking around on those tiles
// * Define the person and the item to be in the room
//     * I think that maybe each of those things should only be optional to define in any given room. Like a room that can only have stuff or only has people should be allowed.
// * Implement picking up the item upon walking over the item
//     * We can implement opening chests later on. If an idea for a generic "thingy" graphic comes up, feel free to replace it, keeping a copy of the chest graphic for later.
//         * An open sack?
// * Is now a good time for an inventory menu?
// * Implement turning in the item and at least like a print when it happens.

// A note about eventual design:
// This bit about Mewgenics having one massive Character class makes me want to support that kind of thing:
// https://www.youtube.com/watch?v=VyxbfbfXzQM&t=764s
// So like we want a way to arbitrarily glue attributes onto a thing, with the config file

// Things to add eventually:
// * A tutorial.
// * A curated list of settings for people to pick for their first several runs.

#[derive(Debug)]
pub enum Error {
    CannotPlacePlayer,
}

impl State {
    pub fn new(seed: Seed, config: Config) -> Result<State, Error> {
        let mut rng = xs::from_seed(seed);

        // TODO a way to enable random rooms in the config
        let world = if false {
            let width = xs::range(&mut rng, 2..9) as SegmentWidth;

            let height = xs::range(&mut rng, 2..9) as usize;

            let len = width * height;
            let mut tiles = Vec::with_capacity(len);

            for _ in 0..len {
                tiles.push(Tile {
                    sprite: xs::range(&mut rng, 0..2) as TileSprite,
                });
            }

            let segment = WorldSegment {
                width,
                tiles,
            };

            World {
                segment,
            }
        } else {
            // TODO? Cap the number of segments, or just be okay with the first room never being in the 5 billions, etc?
            let index = xs::range(&mut rng, 0..config.segments.len() as u32) as usize;

            World {
                segment: config.segments[index].clone(),
            }
        };

        let first_segment = &world.segment;

        let mut player = Entity {
            sprite: models::PLAYER_SPRITE,
            ..<_>::default()
        };

        let xy = random_passable_tile(&mut rng, first_segment)
            .ok_or(Error::CannotPlacePlayer)?;
        player.x = xy.x;
        player.y = xy.y;

        Ok(State {
            rng,
            world,
            player,
            .. <_>::default()
        })
    }
}
