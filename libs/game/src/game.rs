use models::{
    offset,
    Entity,
    Speech,
    Tile,
    X,
    Y,
    XY,
    SegmentId,
    ShakeAmount,
    WorldSegment,
    is_passable,
    xy_to_i,
};
use xs::{Xs, Seed};

use platform_types::{unscaled, arrow_timer::{self, ArrowTimer}};

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
// * Implement the player walking around on those tiles ✔
// * Define the person and the item to be in the room ✔
//     * I think that maybe each of those things should only be optional to define in any given room. Like a room that can only have stuff or only has people should be allowed.
// * Implement picking up the item upon walking over the item ✔
//     * We can implement opening chests later on. If an idea for a generic "thingy" graphic comes up, feel free to replace it, keeping a copy of the chest graphic for later.
//         * An open sack?
// * Is now a good time for an inventory menu?
//    * How should the inventory work?
//        * A grid, with a little description along the bottom?
//            * Implies either like carefully fitting the description into a given space, or some way to scroll. 
//                * Automatic scrolling is an established thing, and we can make the scroll speed adjustable if needed.
//    * Given items are entities, we'd need to support a way to show any entity in the inventory
//        * Maybe they shouldn't be? Or is what we use to display them on the map enough already?
//            * Different items should have a distinct graphic eventually
//    * Let's do a bare bones version of a grid/list and leave stuff like scrolling until later
//        * Press start to bring up the menu, which can be a nine slice box drawn over top of everything ✔
//            * Or maybe just the center tile for it, and we let the edges be ugly for now?
//        * Render the item's tile sprite in a grid, or maybe just a vertical list for now
// * Implement turning in the item and at least like a print when it happens.

// A note about eventual design:
// This bit about Mewgenics having one massive Character class makes me want to support that kind of thing:
// https://www.youtube.com/watch?v=VyxbfbfXzQM&t=764s
// So like we want a way to arbitrarily glue attributes onto a thing, with the config file

// Things to add eventually:
// * A tutorial.
// * A curated list of settings for people to pick for their first several runs.
// * Sources of inspiration for mechanics to throw in the pot:
//    * DROD
//    * All the standard Roguelike things
//    * The standard set of "here's a how to make a game tutorial" easy genres
//    * The thing where you start with people speaking in a weird unitelligble script, then you get letters one by one
//      that transform the script into standard english. I think this is a good fit because you can go out of logic 
//      with some good guesses about what is meant, which is often fun. Can scramble the glyph replacements so players
//      can't learn the alternate script by heart, but maybe allow a setting to keep it the same. And obviously there 
//      should be one to turn this mechanic off.
//      * Eventually can expand this with something more linguistically complex.

pub mod to_tile;

pub mod config {
    use platform_types::{vec1::{Vec1}};
    use models::{DefId, SegmentWidth, Speech, TileSprite};
    pub type TileFlags = u32;

    macro_rules! flags_def {
        (
            $($name: ident = $value: expr),+ $(,)?
        ) => {
            pub const ALL_TILE_FLAGS: [(&str, TileFlags); 5] = [
                $(
                    (stringify!($name), $value),
                )+
            ];

            $(
                pub const $name: TileFlags = $value;
            )+
        };
    }

    flags_def!{
        // Can't be anything but a blocker
        WALL = 0,
        FLOOR = 1 << 0,
        PLAYER_START = 1 << 2,
        ITEM_START = 1 << 3,
        NPC_START = 1 << 4,
    }

    /// A configuration WorldSegment that can be used to contruct game::WorldSegments later.
    #[derive(Clone)]
    pub struct WorldSegment {
        pub width: SegmentWidth,
        // TODO? Nonempty Vec?
        // TODO Since usize is u32 on wasm, let's make a Vec32 type that makes that rsstriction clear, so we
        // can't have like PC only worlds that break in weird ways online. Probably no one will ever need that
        // many tiles per segment. Plus, then xs conversions go away.
        pub tiles: Vec<TileFlags>,
    }

    #[derive(Clone)]
    pub struct Config {
        pub segments: Vec1<WorldSegment>,
        pub entities: Vec1<EntityDef>,
    }

    // Currently not used, and not worth the maintenance cost unless we know it is being used
    //impl Default for Config {
        //fn default() -> Config {
            //const FLOOR: TileFlags = ALL_TILE_FLAGS[1].1;
            //const PLAYER_START: TileFlags = ALL_TILE_FLAGS[2].1;
//
            //Config {
                //segments: vec1![
                    //WorldSegment {
                        //width: 1,
                        //tiles: vec![
                            //FLOOR | PLAYER_START
                        //],
                    //}
                //],
                //entities: vec1![
                    //EntityDef {
                        //kind: EntityDefKind::Mob(()),
                        //speeches: vec![
                            //Speech {
                                //text: format!("hey! would you bring me a specific thing?"),
                            //}
                        //],
                        //id: 0,
                    //},
                    //EntityDef {
                        //kind: EntityDefKind::Item(()),
                        //speeches: vec![],
                        //id: 1,
                    //},
                //],
            //}
        //}
    //}

    pub type EntityDefFlags = u8;

    pub const COLLECTABLE: EntityDefFlags = 1;

    pub const ALL_ENTITY_FLAGS: [(&str, EntityDefFlags); 1] = [
        ("COLLECTABLE", COLLECTABLE),
    ];

    #[derive(Clone, Debug)]
    pub struct EntityDef {        
        pub speeches: Vec<Speech>,
        pub id: DefId,
        pub flags: EntityDefFlags,
        pub tile_sprite: TileSprite,
    }
}
pub use config::{Config, EntityDef, EntityDefFlags, TileFlags, COLLECTABLE};

pub fn to_entity(def: &EntityDef, x: X, y: Y) -> Entity {
    Entity::new(x, y, def.tile_sprite, def.id)
}

mod random {
    use xs::{Xs};
    use crate::config::{self, TileFlags};

    use models::{
        XY,
        i_to_xy,
        is_passable,
        WorldSegment,
    };

    pub fn passable_tile(rng: &mut Xs, segment: &WorldSegment) -> Option<XY> {
        // TODO? Cap tiles length or accept this giving a messed up probabilty for large segments?
        let len = segment.tiles.len();
        let offset = xs::range(rng, 0..len as u32) as usize;
        for index in 0..len {
            let i = (index + offset) % len;
    
            let tile = &segment.tiles[i];
    
            if is_passable(tile) {
                return Some(i_to_xy(segment.width, i));
            }
        }
    
        None
    }

    pub fn tile_matching_flags_besides(
        rng: &mut Xs,
        segment: &config::WorldSegment,
        needle_flags: TileFlags,
        filter_out: &[XY],
    ) -> Option<XY> {
        // TODO? Cap tiles length or accept this giving a messed up probabilty for large segments?
        let len = segment.tiles.len();
        let offset = xs::range(rng, 0..len as u32) as usize;
        for index in 0..len {
            let i = (index + offset) % len;
    
            let current_tile_flags = &segment.tiles[i];
    
            if current_tile_flags & needle_flags == needle_flags {
                let current_xy = i_to_xy(segment.width, i);
                if !filter_out.iter().any(|&xy| current_xy == xy) {
                    return Some(current_xy);
                }
            }
        }
    
        None
    }

    pub fn tile_matching_flags(
        rng: &mut Xs,
        segment: &config::WorldSegment,
        needle_flags: TileFlags
    ) -> Option<XY> {
        tile_matching_flags_besides(
            rng,
            segment,
            needle_flags,
            &[],
        )
    }
}

mod entities {
    use models::{Entity, X, Y, XY, SegmentId};

    use std::collections::{BTreeMap};

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Key {
        pub id: SegmentId,
        pub xy: XY
    }
    
    pub fn entity_key(id: SegmentId, x: X, y: Y) -> Key {
        Key {
            id,
            xy: XY{x, y}
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Entities {
        map: BTreeMap<Key, Entity>,
    }
    
    impl Entities {
        pub fn all_entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
            self.map.values_mut()
        }

        pub fn get(&self, key: Key) -> Option<&Entity> {
            self.map.get(&key)
        }

        pub fn for_id(&self, id: SegmentId) -> impl Iterator<Item=(&Key, &Entity)> {
            self.map.range(entity_key(id, X::MIN, Y::MIN)..=entity_key(id, X::MAX, Y::MAX))
        }

        pub fn insert(&mut self, id: SegmentId, entity: Entity) -> Option<Entity> {
            self.map.insert(entity_key(id, entity.x, entity.y), entity)
        }

        pub fn remove(&mut self, key: Key) -> Option<Entity> {
            self.map.remove(&key)
        }
    }

    #[cfg(test)]
    mod entities_works {
        use models::{xy::{x, y}};
        use super::*;
    
        #[test]
        fn when_pulling_out_this_range() {
            let mut entities = Entities::default();
    
            let id = 0;
    
            let mut a = Entity::default();
            a.x = x(1);
            a.y = y(2);
    
            let mut b = Entity::default();
            b.x = x(3);
            b.y = y(3);
    
            let mut c = Entity::default();
            c.x = x(1);
            c.y = y(2);
    
            entities.insert(id, a.clone());
            entities.insert(id, b.clone());
            entities.insert(id + 1, c);
    
            let mut actual = vec![];
    
            for (_, v) in entities.for_id(id) {
                actual.push(v.clone());
            }
    
            let expected = vec![a, b];
    
            assert_eq!(actual, expected);
        }
    }
}
use entities::{Entities, entity_key};

mod speeches {
    use models::{DefId, Speech};

    pub type Key = DefId;

    #[derive(Clone, Debug, Default)]
    pub struct Speeches {
        // For now, it seems reasonable to assume we can force Def IDs to be dense, and start at 0.
        // TODO: Non-empty vecs?
        speeches: Vec<Vec<Speech>>,
    }
    
    impl Speeches {
        pub fn new(speeches: Vec<Vec<Speech>>) -> Self {
            Self {
                speeches,
            }
        }

        pub fn get(&self, key: Key) -> Option<&[Speech]> {
            self.speeches.get(key as usize).map(|v| &**v)
        }
    }
}
use speeches::{Speeches};

#[derive(Clone, Default)]
pub struct World {
    // TODO a graph structure of `WorldSegment`s instead of just one
    pub segment: WorldSegment,
    pub items: Entities,
    pub mobs: Entities,
}

impl World {
    pub fn all_entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.items.all_entities_mut().chain(self.mobs.all_entities_mut())
    }

    pub fn get_entity(&self, key: entities::Key) -> Option<&Entity> {
        self.mobs.get(key).or_else(|| self.items.get(key))
    }
}

fn can_walk_onto(world: &World, id: SegmentId, x: X, y: Y) -> bool {
    let Ok(i) = xy_to_i(&world.segment, x, y) else {
        return false;
    };

    if let Some(tile) = &world.segment.tiles.get(i) {
        let key = entity_key(id, x, y);

        if let Some(_) = world.mobs.get(key) {
            return false;
        }

        if is_passable(tile) {
            return true;
        }
    }

    false
}

pub type Inventory = Vec<Entity>;

/// 64k speech boxes ought to be enough for anybody!
pub type SpeechIndex = u16;

#[derive(Clone)]
pub struct TalkingState {
    pub key: entities::Key,
    pub speech_index: SpeechIndex,
    pub arrow_timer: ArrowTimer,
}

impl TalkingState {
    pub fn new(key: entities::Key) -> Self {
        Self {
            key,
            speech_index: <_>::default(),
            arrow_timer: <_>::default(),
        }
    }
}

#[derive(Clone, Default)]
pub enum Mode {
    #[default]
    Walking,
    Inventory {},
    Talking(TalkingState),
}

/// 64k fade frames ought to be enough for anybody!
type FadeTimer = u16;

#[derive(Clone)]
pub struct FadeMessage {
    pub text: String,
    pub fade_timer: FadeTimer,
    pub xy: unscaled::XY,
    pub wh: unscaled::WH,
}

impl FadeMessage {
    pub fn new(text: String, xy: XY) -> Self {
        Self {
            text,
            // TODO? Scale this based on text length?
            fade_timer: 100,
            xy: to_tile::center(xy),
            // TODO? Scale this based on text length?
            wh: unscaled::WH { w: unscaled::W::ZERO, h: unscaled::H::ONE },
        }
    }
}

// TODO? Put a hard limit on the amount of these, with I guess LIFO eviction?
pub type FadeMessages = Vec<FadeMessage>;

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub world: World,
    pub player: Entity,
    pub player_inventory: Inventory,
    pub segment_id: SegmentId,
    pub mode: Mode,
    pub fade_messages: FadeMessages,
    pub shake_amount: ShakeAmount,
    pub speeches: Speeches,
}

impl State {
    pub fn all_entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.world.all_entities_mut().chain(std::iter::once(&mut self.player))
    }

    pub fn get_entity(&self, key: entities::Key) -> Option<&Entity> {
        let player_key = entity_key(
            self.segment_id,
            self.player.x,
            self.player.y,
        );

        if key == player_key {
            return Some(&self.player)
        }

        self.world.get_entity(key)
    }
}

#[derive(Debug)]
pub enum Error {
    CannotPlacePlayer,
    NoMobsFound,
    NoItemsFound,
}

impl State {
    pub fn new(seed: Seed, config: Config) -> Result<State, Error> {
        use config::{FLOOR, PLAYER_START, NPC_START, ITEM_START};

        let mut next_free_segment_id = 0;

        let mut new_segment_id = || {
            let output = next_free_segment_id;
            next_free_segment_id += 1;
            output
        };

        let mut rng = xs::from_seed(seed);

        // TODO? Cap the number of segments, or just be okay with the first room never being in the 5 billions, etc?
        let index = xs::range(&mut rng, 0..config.segments.len() as u32) as usize;

        let config_segment = &config.segments[index];

        let tiles = config_segment.tiles.iter().map(
            |tile_flags| {
                Tile {
                    sprite: if tile_flags & FLOOR != 0 {
                        models::FLOOR_SPRITE
                    } else {
                        models::WALL_SPRITE
                    },
                }
            }
        ).collect();

        let segment = WorldSegment {
            id: new_segment_id(),
            width: config_segment.width,
            tiles,
        };

        let mut world = World {
            segment,
            items: <_>::default(),
            mobs: <_>::default(),
        };

        let first_segment = &world.segment;

        let mut player = Entity {
            sprite: models::PLAYER_SPRITE,
            ..<_>::default()
        };

        let p_xy = random::tile_matching_flags(&mut rng, &config_segment, PLAYER_START)
            .or_else(|| random::passable_tile(&mut rng, first_segment))
            .ok_or(Error::CannotPlacePlayer)?;
        player.x = p_xy.x;
        player.y = p_xy.y;

        let mut mob_defs = Vec::with_capacity(16);
        let mut item_defs = Vec::with_capacity(16);
        let mut speeches_lists = Vec::with_capacity(16);

        for def in &config.entities {
            if def.flags & COLLECTABLE == COLLECTABLE {
                item_defs.push(def);
            } else {
                mob_defs.push(def);
            }

            // PERF: Is it worth it to avoid this clone?
            speeches_lists.push(def.speeches.clone());
        }

        // TODO? Non-empty Vec
        if mob_defs.is_empty() {
            return Err(Error::NoMobsFound);
        }

        if item_defs.is_empty() {
            return Err(Error::NoItemsFound);
        }

        xs::shuffle(&mut rng, &mut mob_defs);
        xs::shuffle(&mut rng, &mut item_defs);

        let speeches = Speeches::new(speeches_lists);

        let mut placed_already = Vec::with_capacity(16);
        placed_already.push(p_xy);

        while let Some(npc_xy) = random::tile_matching_flags_besides(
            &mut rng,
            &config_segment,
            ITEM_START,
            &placed_already,
        ) {
            // FIXME: Actual logic to keep things solvable.

            if let Some(mob_def) = mob_defs.pop() {
                world.mobs.insert(
                    first_segment.id,
                    to_entity(mob_def, npc_xy.x, npc_xy.y),
                );
                placed_already.push(npc_xy);

                // TODO? probably combine all these option checks into one match?
                if let Some(item_xy) = random::tile_matching_flags_besides(
                    &mut rng,
                    &config_segment,
                    NPC_START,
                    &placed_already,
                ) {
                    if let Some(item_def) = item_defs.pop() {
                        world.items.insert(
                            first_segment.id,
                            to_entity(item_def, item_xy.x, item_xy.y),
                        );
                        placed_already.push(item_xy);
                    }
                }
            } else {
                break
            }
        }

        Ok(State {
            rng,
            world,
            player,
            speeches,
            .. <_>::default()
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Dir {
    Left,
    Right,
    Up,
    Down,
}

fn xy_in_dir(x: X, y: Y, dir: Dir) -> Option<XY> {
    use Dir::*;

    let (new_x, new_y) = match dir {
        Left => (x.dec(), y),
        Right => (x.inc(), y),
        Up => (x, y.dec()),
        Down => (x, y.inc()),
    };

    // This can happen due to saturation
    if new_x == x
    && new_y == y {
        return None
    }

    Some(XY { x: new_x, y: new_y })
}

pub fn get_speech(state: &State, key: entities::Key, speech_index: SpeechIndex) -> Option<&Speech> {
    state.get_entity(key)
        .and_then(|entity| 
            state.speeches.get(entity.def_id)
                .and_then(|list| list.get(speech_index as usize))
        )
}

impl State {
    pub fn walk(&mut self, dir: Dir) {
        let entity = &mut self.player;

        let Some(XY { x: new_x, y: new_y }) = xy_in_dir(entity.x, entity.y, dir) else {
            return
        };

        if can_walk_onto(&self.world, self.segment_id, new_x, new_y) {
            // TODO? Worth making every update to any entities x/y update the offset?
            entity.offset_x = offset::X::from(entity.x) - offset::X::from(new_x);
            entity.offset_y = offset::Y::from(entity.y) - offset::Y::from(new_y);

            entity.x = new_x;
            entity.y = new_y;

            let key = entity_key(self.segment_id, entity.x, entity.y);

            if let Some(item) = self.world.items.remove(key) {
                // Mostly for testing putposes until we get to combat or other things that make sense to cause 
                // screenshake
                self.shake_amount = 5;

                self.player_inventory.push(item);
            }
        }
    }

    pub fn interact(&mut self, dir: Dir) {
        let entity = &mut self.player;

        let Some(XY { x: target_x, y: target_y }) = xy_in_dir(entity.x, entity.y, dir) else {
            self.fade_messages.push(FadeMessage::new(format!("there's nothing there."), entity.xy()));
            return
        };

        let key = entity_key(self.segment_id, target_x, target_y);

        let Some(mob) = self.world.mobs.get(key) else {
            self.fade_messages.push(
                FadeMessage::new(format!("there's nobody there."), entity.xy())
            );
            return
        };

        if self.speeches.get(mob.def_id).is_some() {
            self.mode = Mode::Talking(TalkingState::new(key));
            return
        }

        self.fade_messages.push(
            FadeMessage::new(
                format!(
                    "what do you want me to do with {}?",
                    models::entity_article_phrase(mob),
                ),
                entity.xy()
            )
        );
    }

    pub fn tick(&mut self) {
        //
        // Advance Timers
        //
        for i in (0..self.fade_messages.len()).rev() {
            let message = &mut self.fade_messages[i];

            message.fade_timer = message.fade_timer.saturating_sub(1);
            if message.fade_timer == 0 {
                self.fade_messages.remove(i);
                continue
            }

            // TODO? A timer or other method to be able to move less than one pixel per frame?
            //     At that point, do we want sub-pixel blending enough to implement it?
            message.xy += message.wh;
        }

        match &mut self.mode {
            Mode::Talking(talking) => {
                arrow_timer::tick(&mut talking.arrow_timer);
            }
            Mode::Walking
            | Mode::Inventory {} => {
                // No timers
            }
        }

        match &self.mode {
            Mode::Talking(talking) => {
                if get_speech(self, talking.key, talking.speech_index).is_none() {
                    self.mode = Mode::Walking;
                }
            }
            Mode::Walking
            | Mode::Inventory {} => {
                // No timers
            }
        }

        
                

        // The offests are timers of a sort.
        for entity in self.all_entities_mut() {
            /// Distinct from f32::signum in that it returns 0.0 for 0.0, -0.0, NaNs, etc.
            fn sign(x: f32) -> f32 {
                if x > 0.0{
                    1.0
                } else if x < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            }

            const DECAY_RATE: f32 = 1./8.;

            entity.offset_x -= sign(entity.offset_x) * DECAY_RATE;
            entity.offset_y -= sign(entity.offset_y) * DECAY_RATE;
        }
    }
}