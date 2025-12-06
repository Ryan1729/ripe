use models::{
    offset,
    speeches,
    CollectAction,
    DefId,
    Entity,
    Speech,
    Speeches,
    Tile,
    TileSprite,
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

use platform_types::{arrow_timer::{ArrowTimer}, Vec1};

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
//        * So like a random hallway with like one monster in it, for now
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
// * Is now a good time for an inventory menu? ✔
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
// * Implement turning in the item and at least like a print when it happens. ✔
//    * Need to describe the connection between items and NPCs in the config file
//        * Hmm. Could theoretically make it purely random, but I feel like allowing some structure here is more useful
//          to config authors.
//            * I guess having a way to make things purely random would make sesne. Maybe by leaving off the connection
//              annotaion described below?
//        * Best way, given the implicit ID structure, seems to be to allow saying "The previous mob wants the previous item"
//            * `wanted_by: relative(-1)`
//                * So an absolute reference would be possible. We can have `relative` return a map with a kind field 
//                  to check during parsing.
//            * What about making the mob be marked as wanting the item? I don't think it makes a big difference
//              to the parsing code, but I expect it might be more inuitive that way when writing the config.
//                * `wants: relative(-1)`
//    * Will also need a way to actually win the whole run, and  thus a way to determine that from the config file.
//        * I like the idea of a random goal as an option. Like this time they just really want some donuts. 
//          But then this time they need the secret codes to overthrow the facist government, or the last known 
//          wherabouts of the person they are investigating the disappearance of, or their mom wants the donuts.
//        * Should also allow "triforce hunt" runs, where it's get N of M of a certain kind of item.
//        * So an optional field on the root that indicates the goal I guess? Probably needs a specific GoalSpec type.
//            * We will eventually want to have boss fights or "beat this minigame" as goals.
//            * I guess minigame entrances can be represented as entities?
//                * So maybe instead of a COLLECTABLE flag, it's a "steppable" flag, and the things have a 
//                  "when_stepped_on" field that can be "collect" vs "start minigame"?
//                    * Maybe doors can be placed on walls, and then still be "steppable"? Or does that 
//                      complicate things, and they should just embed a wall into the sprite if they want
//                      to look like that? Feels like the second one.
//        * I think the goal for this next pass should be to get to a random item is on the ground, than one npc wants,
//          that then gives you what another NPC wants, and that NPC gives you the thing that you want, and you win!
//        * Concrete steps then
//            * Add a victory colectable sprite that makes some sense to be a victory when walked over ✔
//                * A portal?
//            * Mark it as instant_victory in the config file ✔
//            * Have it actually trigger a victory screen ✔
//                * Worth adding a walking through the door animation? Like 3 frames or whatever?
//            * Is it a good time to add an in-game display of the goal? ✔
//                * Let's do at least the most basic version: some text in the pause menu
//            * Add a locked door/portal sprite ✔
//            * Add a key sprite ✔
//            * Ensure the key spawns
//                * Add to config as new item entry
//            * Have the portal start locked
//                * Spawn only the locked door at the start
//                    * Have a "NOT_SPAWNED_AT_START" flag that defaults to false
//                        * Less work than marking all the other entities
//            * Make collecting the key open the corresponding door
//                * Allow it to work for multiple doors in the future
//                    * Markup key as transforming instances of one entity ID into another
//            * Put items in the NPC's pockets, and have them actually give them to you when you give them a thing
//            * Once it has been shown to work for one door, actually make multiple doors

// A note about eventual design:
// This bit about Mewgenics having one massive Character class makes me want to support that kind of thing:
// https://www.youtube.com/watch?v=VyxbfbfXzQM&t=764s
// So like we want a way to arbitrarily glue attributes onto a thing, with the config file

// Things to add eventually:
// * A tutorial.
// * An in-game display of the goal
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
//    * Idleon seems to have a bunch of minigames, that are already based around getting a reward

pub mod config {
    use platform_types::{vec1::{Vec1}};
    use models::{DefId, OnCollect, SegmentWidth, Speech, TileSprite};
    use crate::MiniEntityDef;

    macro_rules! consts_def {
        (
            $all_name: ident : $type: ty;
            $($name: ident = $value: expr),+ $(,)?
        ) => {
            

            pub const $all_name: [(&str, $type); const {
                let mut count = 0;

                $(
                    // Use the repetition for something so we can take the count
                    const _: $type = $value;
                    count += 1;
                )+

                count
            }] = [
                $(
                    (stringify!($name), $value),
                )+
            ];

            $(
                pub const $name: $type = $value;
            )+
        };
    }

    pub type TileFlags = u32;

    consts_def!{
        ALL_TILE_FLAGS: TileFlags;
        // Can't be anything but a blocker
        WALL = 0,
        FLOOR = 1 << 0,
        PLAYER_START = 1 << 2,
        ITEM_START = 1 << 3,
        NPC_START = 1 << 4,
        DOOR_START = 1 << 5,
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

    pub type EntityDefFlags = u8;

    consts_def!{
        ALL_ENTITY_FLAGS: EntityDefFlags;
        COLLECTABLE = 1 << 0,
        STEPPABLE = 1 << 1,
        VICTORY = 1 << 2,
        NOT_SPAWNED_AT_START = 1 << 3,
    }

    pub type EntityDefIdRefKind = u8;

    consts_def!{
        ALL_ENTITY_ID_REFERENCE_KINDS: EntityDefIdRefKind;
        RELATIVE = 1,
        ABSOLUTE = 2,
    }

    pub type CollectActionKind = u8;

    consts_def!{
        ALL_COLLECT_ACTION_KINDS: CollectActionKind;
        TRANSFORM = 1,
    }

    #[derive(Clone, Debug)]
    pub struct EntityDef {
        pub speeches: Vec<Vec<Speech>>,
        pub inventory_description: Vec<Vec<Speech>>,
        pub id: DefId,
        pub flags: EntityDefFlags,
        pub tile_sprite: TileSprite,
        pub wants: Vec<DefId>,
        pub on_collect: OnCollect,
    }

    #[derive(Clone, Debug)]
    pub struct Desire {
        pub mob_def: MiniEntityDef,
        pub item_def: MiniEntityDef,
    }

    impl Desire {
        pub fn borrow(&self) -> DesireRef<'_> {
            DesireRef {
                mob_def: &self.mob_def,
                item_def: &self.item_def,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct DesireRef<'defs> {
        pub mob_def: &'defs MiniEntityDef,
        pub item_def: &'defs MiniEntityDef,
    }

    impl <'defs> DesireRef<'defs> {
        pub fn to_owned(&self) -> Desire {
            Desire {
                mob_def: self.mob_def.to_owned(),
                item_def: self.item_def.to_owned(),
            }
        }
    }
}
pub use config::{Config, EntityDef, EntityDefFlags, TileFlags, COLLECTABLE, STEPPABLE, VICTORY, NOT_SPAWNED_AT_START};

pub fn to_entity(
    def: &MiniEntityDef,
    x: X,
    y: Y
) -> Entity {
    let mut flags = 0;

    if def.flags & config::COLLECTABLE == config::COLLECTABLE {
        flags |= models::COLLECTABLE;
    }

    if def.flags & config::VICTORY == config::VICTORY {
        flags |= models::VICTORY;
    }

    Entity::new(
        x,
        y,
        def.tile_sprite,
        def.id,
        def.wants.iter().map(|&def_id| models::Desire::new(def_id)).collect(),
        def.on_collect.clone(),
        flags,
    )
}

fn transform_entity(entity: &mut Entity, def: &MiniEntityDef) {
    // TODO? Is there anything else we'd want to keep during transformations besides position?
    // TODO? Is it worth storing pre-processed entity Defs, instead of the whole thing? In terms of any of
    //       reduced memory usage, less work needed to do these transforms, or reduced mixing of concerns?
    *entity = to_entity(
        def,
        entity.x,
        entity.y
    );
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

        pub fn get_mut(&mut self, key: Key) -> Option<&mut Entity> {
            self.map.get_mut(&key)
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
                actual.push(v.xy());
            }
    
            let expected = vec![a.xy(), b.xy()];
    
            assert_eq!(actual, expected);
        }
    }
}
use entities::{Entities, entity_key};

#[derive(Clone, Default)]
pub struct World {
    // TODO a graph structure of `WorldSegment`s instead of just one
    pub segment: WorldSegment,
    /// The ID of the current segment we are in.
    pub segment_id: SegmentId,
    pub player: Entity,
    pub steppables: Entities,
    pub mobs: Entities,
}

impl World {
    pub fn all_entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        std::iter::once(&mut self.player).chain(self.steppables.all_entities_mut().chain(self.mobs.all_entities_mut()))
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

        self.mobs.get(key).or_else(|| self.steppables.get(key))
    }

    pub fn get_entity_mut(&mut self, key: entities::Key) -> Option<&mut Entity> {
        let player_key = entity_key(
            self.segment_id,
            self.player.x,
            self.player.y,
        );

        if key == player_key {
            return Some(&mut self.player)
        }

        self.mobs.get_mut(key).or_else(|| self.steppables.get_mut(key))
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

#[derive(Clone, Copy, Debug, Default)]
pub enum PostTalkingAction {
    #[default]
    NoOp,
    TakeItem(entities::Key, DefId),
}

#[derive(Clone, Debug)]
pub struct TalkingState {
    pub key: speeches::Key,
    pub speech_index: SpeechIndex,
    pub arrow_timer: ArrowTimer,
    pub post_action: PostTalkingAction,
}

impl TalkingState {
    pub fn new(key: speeches::Key) -> Self {
        Self::new_with_action(key, <_>::default())
    }

    pub fn new_with_action(
        key: speeches::Key,
        post_action: PostTalkingAction,
    ) -> Self {
        Self {
            key,
            speech_index: <_>::default(),
            arrow_timer: <_>::default(),
            post_action,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub enum Mode {
    #[default]
    Walking,
    Inventory {
        current_index: usize,
        last_dir: Option<Dir>,
        dir_count: u8,
        description_talking: Option<TalkingState>,
    },
    Talking(TalkingState),
    Victory(DoorAnimation),
}

#[derive(Clone)]
pub struct FadeMessageSpec {
    pub message: String,
    pub xy: XY,
}

impl FadeMessageSpec {
    pub fn new(message: String, xy: XY) -> Self {
        Self {
            message,
            xy,
        }
    }
}

// TODO? Put a hard limit on the amount of these? Like this could perhaps be just an Option?
pub type FadeMessageSpecs = Vec<FadeMessageSpec>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DoorAnimation {
    frame: u16
}

impl DoorAnimation {
    pub fn advance_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub fn is_done(&self) -> bool {
        self.frame >= 150
    }

    // TODO: reference the config file somehow to determine this.
    //     Probably copy the frames somewhere, instead of referencing it every frame
    pub fn sprite(&self) -> models::TileSprite {
        match self.frame {
            x if x < 50 => models::DOOR_ANIMATION_FRAME_1,
            x if x < 100 => models::DOOR_ANIMATION_FRAME_2,
            _ => models::DOOR_ANIMATION_FRAME_3,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MiniEntityDef {
    pub id: DefId,
    pub flags: EntityDefFlags,
    pub tile_sprite: TileSprite,
    pub wants: Vec<DefId>,
    pub on_collect: models::OnCollect,
}

impl From<&EntityDef> for MiniEntityDef {
    fn from(def: &EntityDef) -> Self {
        Self {
            id: def.id,
            flags: def.flags,
            tile_sprite: def.tile_sprite,
            wants: def.wants.clone(),
            on_collect: def.on_collect.clone(),
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub rng: Xs,
    pub world: World,
    pub player_inventory: Inventory,
    pub mode: Mode,
    pub fade_message_specs: FadeMessageSpecs,
    pub shake_amount: ShakeAmount,
    // Fairly direct from the config section {
    // Is the lookup acceleration, reduced memory usage etc. worth the extra code vs 
    // just storing the config here directly?
    pub speeches: Speeches,
    pub inventory_descriptions: Speeches,
    pub entity_defs: Vec1<MiniEntityDef>,
    // }
}

impl State {
    pub fn all_entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.world.all_entities_mut()
    }
}

#[derive(Debug)]
pub enum Error {
    CannotPlacePlayer,
    CannotPlaceDoor,
    NoEntityDefs,
    NoMobsFound,
    NoItemsFound,
    NoDoorsFound,
    CouldNotSatisfyDesire(config::Desire),
    InvalidDesireID(MiniEntityDef, SegmentId),
    NonItemWasDesired(MiniEntityDef, MiniEntityDef, SegmentId),
    InvalidSpeeches(speeches::PushError),
    InvalidInventoryDescriptions(speeches::PushError),
}

impl State {
    pub fn new(seed: Seed, config: Config) -> Result<State, Error> {
        use config::{FLOOR, DOOR_START, PLAYER_START, NPC_START, ITEM_START};

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

        let player = Entity {
            sprite: models::PLAYER_SPRITE,
            ..<_>::default()
        };

        let mut world = World {
            player,
            segment_id: segment.id,
            segment,
            steppables: <_>::default(),
            mobs: <_>::default(),
        };

        let first_segment = &world.segment;

        let p_xy = random::tile_matching_flags(&mut rng, &config_segment, PLAYER_START)
            .or_else(|| random::passable_tile(&mut rng, first_segment))
            .ok_or(Error::CannotPlacePlayer)?;
        world.player.x = p_xy.x;
        world.player.y = p_xy.y;

        let mut placed_already = Vec::with_capacity(16);
        placed_already.push(p_xy);

        let mut mob_defs = Vec::with_capacity(16);
        let mut item_defs = Vec::with_capacity(16);
        let mut door_defs = Vec::with_capacity(16);
        let mut speeches_lists = Vec::with_capacity(16);
        let mut inventory_descriptions_lists = Vec::with_capacity(16);

        for def in &config.entities {
            // PERF: Is it worth it to avoid this clone?
            speeches_lists.push(def.speeches.clone());
            inventory_descriptions_lists.push(def.inventory_description.clone());
        }

        let mut speeches = Speeches::with_capacity(speeches_lists.len());
        for list in &mut speeches_lists {
            speeches.push(list).map_err(Error::InvalidSpeeches)?;
        }
        let mut inventory_descriptions = Speeches::with_capacity(inventory_descriptions_lists.len());
        for list in &mut inventory_descriptions_lists {
            inventory_descriptions.push(list).map_err(Error::InvalidInventoryDescriptions)?;
        }

        let mut all_desires = Vec::with_capacity(
            core::cmp::min(
                // A loose upper bound
                config.entities.len() / 2,
                1024,
            )
        );

        let mut entity_defs = Vec::<MiniEntityDef>::with_capacity(config.entities.len());

        for i in 0..config.entities.len() {
            let def = &config.entities[i];

            entity_defs.push(def.into());
        }

        // Don't shuffle this one! We use it later expecting def_id to be indexes!
        let entity_defs: Vec1<_> = entity_defs.try_into().map_err(|_| Error::NoEntityDefs)?;

        for def in &entity_defs {
            if def.flags & COLLECTABLE == COLLECTABLE {
                item_defs.push(def.clone());
            } else if def.flags & STEPPABLE == STEPPABLE {
                door_defs.push(def.clone());
            } else {
                mob_defs.push(def.clone());

                for &wanted_id in &def.wants {
                    if let Some(desired_def) = entity_defs.get(wanted_id.into()) {
                        if desired_def.flags & COLLECTABLE == COLLECTABLE {
                            all_desires.push(config::DesireRef {
                                mob_def: def,
                                item_def: desired_def,
                            });
                        } else {
                            return Err(Error::NonItemWasDesired(def.clone(), desired_def.clone(), wanted_id))
                        }
                    } else {
                        return Err(Error::InvalidDesireID(def.clone(), wanted_id))
                    }
                }
            }
        }

        // TODO? Non-empty Vec
        if mob_defs.is_empty() {
            return Err(Error::NoMobsFound);
        }

        if item_defs.is_empty() {
            return Err(Error::NoItemsFound);
        }

        if door_defs.is_empty() {
            return Err(Error::NoDoorsFound);
        }

        xs::shuffle(&mut rng, &mut mob_defs);
        xs::shuffle(&mut rng, &mut item_defs);
        xs::shuffle(&mut rng, &mut door_defs);

        // TEMP: Get the key to spawn for testing. Later, it should be given as a reward
        for item_def in &item_defs {
            if item_def.on_collect.is_empty() { continue }

            let key_xy = random::tile_matching_flags_besides(
                &mut rng,
                &config_segment,
                ITEM_START,
                &placed_already,
            ).ok_or(Error::CannotPlaceDoor)?;

            world.steppables.insert(
                first_segment.id,
                to_entity(
                    item_def,
                    key_xy.x,
                    key_xy.y
                ),
            );

            placed_already.push(key_xy);
        }

        for door_def in &door_defs {
            if door_def.flags & NOT_SPAWNED_AT_START == NOT_SPAWNED_AT_START {
                continue
            }

            let d_xy = random::tile_matching_flags_besides(
                &mut rng,
                &config_segment,
                FLOOR | DOOR_START,
                &placed_already,
            ).ok_or(Error::CannotPlaceDoor)?;
    
            world.steppables.insert(
                first_segment.id,
                to_entity(
                    door_def,
                    d_xy.x,
                    d_xy.y
                ),
            );

            placed_already.push(d_xy);
        }

        struct Constraints<'desires> {
            desires: Vec<config::DesireRef<'desires>>,
        }

        // Select a random subset of the desires to make into constriants
        fn select_constraints<'defs>(rng: &mut Xs, all_desires: &[config::DesireRef<'defs>]) -> Constraints<'defs> {
            let target_len = xs::range(rng, 1..all_desires.len() as u32 + 1) as usize;
            let initial_index = xs::range(rng, 0..all_desires.len() as u32) as usize;

            let mut desires: Vec<_> = Vec::with_capacity(target_len);

            let mut index = initial_index;
            while desires.len() < target_len {
                // Select the index or not, at a rate proportional to how many we need.
                if (xs::range(rng, 0..all_desires.len() as u32 + 1) as usize) < target_len {
                    desires.push(all_desires[index]);
                }

                index += 1;
                if index >= all_desires.len() {
                    index = 0;
                }

                if index == initial_index {
                    // Avoid using the value from any index more than once.
                    break
                }
            }

            Constraints {
                desires,
            }
        }

        let constraints: Constraints = select_constraints(&mut rng, &all_desires);

        for desire in constraints.desires {
            let mut attempts = 0;

            while attempts < 16 {
                attempts += 1;

                // TODO? Is there a nicer way to do this nested checking, and handle `placed_already`?
                if let Some(npc_xy) = random::tile_matching_flags_besides(
                    &mut rng,
                    &config_segment,
                    NPC_START,
                    &placed_already,
                ) {
                    placed_already.push(npc_xy);

                    if let Some(item_xy) = random::tile_matching_flags_besides(
                        &mut rng,
                        &config_segment,
                        ITEM_START,
                        &placed_already,
                    ) {
                        world.mobs.insert(
                            first_segment.id,
                            to_entity(
                                &desire.mob_def,
                                npc_xy.x,
                                npc_xy.y
                            ),
                        );

                        world.steppables.insert(
                            first_segment.id,
                            to_entity(
                                &desire.item_def,
                                item_xy.x,
                                item_xy.y
                            ),
                        );
                        placed_already.push(item_xy);

                        break
                    } else {
                        placed_already.pop();
                    }
                }
            }

            if attempts >= 16 {
                return Err(Error::CouldNotSatisfyDesire(desire.to_owned()));
            }
        }

        Ok(State {
            rng,
            world,
            player_inventory: <_>::default(),
            mode: <_>::default(),
            fade_message_specs: <_>::default(),
            shake_amount: <_>::default(),
            speeches,
            inventory_descriptions,
            entity_defs,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub fn get_speech<'speeches>(speeches: &'speeches Speeches, key: speeches::Key, speech_index: SpeechIndex) -> Option<&'speeches Speech> {
    speeches.get(key)
        .and_then(|list| list.get(speech_index as usize))
}

impl State {
    pub fn walk(&mut self, dir: Dir) {
        let Some(XY { x: new_x, y: new_y }) = xy_in_dir(self.world.player.x, self.world.player.y, dir) else {
            return
        };

        if can_walk_onto(&self.world, self.world.segment_id, new_x, new_y) {
            // TODO? Worth making every update to any entities x/y update the offset?
            self.world.player.offset_x = offset::X::from(self.world.player.x) - offset::X::from(new_x);
            self.world.player.offset_y = offset::Y::from(self.world.player.y) - offset::Y::from(new_y);

            self.world.player.x = new_x;
            self.world.player.y = new_y;

            let key = entity_key(self.world.segment_id, self.world.player.x, self.world.player.y);

            if let Some(steppable) = self.world.steppables.remove(key) {
                // Mostly for testing purposes until we get to combat or other things that make sense to cause 
                // screenshake
                self.shake_amount = 5;

                if steppable.is_collectable() {
                    for action in &steppable.on_collect {
                        match action {
                            CollectAction::Transform { from, to } => {
                                if let Some(to_def) = self.entity_defs.get((*to) as usize) {
                                    for entity in self.world.all_entities_mut() {
                                        if entity.def_id == *from {
                                            transform_entity(entity, to_def);
                                        }
                                    }
                                } else {
                                    debug_assert!(false, "Why are we trying to transform something into something that doesn't exist? to {to}");
                                }
                            }
                        }
                    }
                    self.player_inventory.push(steppable);
                } else if steppable.is_victory() {
                    self.mode = Mode::Victory(<_>::default());
                } else {
                    // Effectively just disappearing scenery. Could make this not go away if we have a reason to.
                }
            }
        }
    }

    #[must_use]
    pub fn interact(&mut self, dir: Dir) {
        let entity = &mut self.world.player;

        let Some(XY { x: target_x, y: target_y }) = xy_in_dir(entity.x, entity.y, dir) else {
            self.fade_message_specs.push(FadeMessageSpec::new(format!("there's nothing there."), entity.xy()));
            return
        };

        let key = entity_key(self.world.segment_id, target_x, target_y);

        let Some(mob) = self.world.mobs.get_mut(key) else {
            self.fade_message_specs.push(
                FadeMessageSpec::new(format!("there's nobody there."), entity.xy())
            );
            return
        };

        let mut post_action = PostTalkingAction::NoOp;
        for desire in &mut mob.desires {
            use models::DesireState::*;
            // Check if mob should notice the player's item.
            if desire.state == Unsatisfied 
            && self.player_inventory.iter().any(|e| e.def_id == desire.def_id) {
                desire.state = SatisfactionInSight;
                post_action = PostTalkingAction::TakeItem(key, desire.def_id);
            }
        }

        let speeches_key = mob.speeches_key();

        if let Some(speeches) = self.speeches.get(speeches_key) {
            if !speeches.is_empty() {
                self.mode = Mode::Talking(TalkingState::new_with_action(speeches_key, post_action));
                return
            }
        }

        self.fade_message_specs.push(
            FadeMessageSpec::new(
                format!(
                    "what do you want me to do with {}?",
                    models::entity_article_phrase(mob),
                ),
                entity.xy()
            )
        );
    }

    pub fn tick(&mut self) {
        match &mut self.mode {
            Mode::Victory(animation) => {
                animation.advance_frame();
            },
            Mode::Walking => { /* fall through to rest of method */ }
            Mode::Inventory { .. }
            | Mode::Talking(..) => return,
        }

        if ! matches!(self.mode, Mode::Walking) {
            return
        }

        //
        // Advance Timers
        //

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