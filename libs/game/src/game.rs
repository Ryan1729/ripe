use models::{Card, gen_card};
use platform_types::{command, unscaled};
use xs::{Xs, Seed};

pub struct Config {
    
}

type TileSprite = u8;

#[derive(Clone, Default)]
pub struct Tile {
    pub sprite: TileSprite,
}

pub type SegmentWidth = usize;

#[derive(Clone, Default)]
pub struct WorldSegment {
    pub width: SegmentWidth,
    pub tiles: Vec<Tile>,
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub segment: WorldSegment,
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

// Substeps of "Go find a thing and bring it to the person who wants it"
// * Define an example of a text format, which can define at least one room's tiles for now, with room for expansion
//    * Lets use TOML, mostly for comments.
//    * Always include a version number!
//    * Include a required way to define where the player starts, if they start in that room
// * Embed a string of that format in the program for now.
// * Parse that string into the definition of the tiles
//    * Leave room for a validate step after the parsing. Validation errors should eventually all contain custom error messages including why the given thing is needed.
// * Implement the player walking around on those tiles
// * Define the person and the item to be in the room
//     * I think that maybe each of those things should only be optional to define in any given room. Like a room that can only have stuff or only has people should be allowed.
// * Implement picking up the item upon walking over the item
//     * We can implement opening chests later on. If an idea for a generic "thingy" graphic comes up, feel free to repalce it, keeping a copy of the chest graphic for later.
//         * An open sack?
// * Is now a good time for an inventory menu?
// * Implement turning in the item and at least like a print when it happens.

impl State {
    pub fn new(seed: Seed) -> State {
        let mut rng = xs::from_seed(seed);

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

        State {
            rng,
            segment,
            .. <_>::default()
        }
    }
}
