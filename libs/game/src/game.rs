use models::{
    offset,
    speeches,
    CollectAction,
    DefId,
    Entity,
    Location,
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
type DoorTarget = Location;

use xs::{Xs, Seed};

use platform_types::{arrow_timer::{ArrowTimer}, Vec1, vec1::vec1};

// Proposed Steps
// * Make the simplest task: Go find a thing and bring it to the person who wants it ✔
//     * I think baking in things being parsed from a file early, makes sense
//         * Can start with an embedded string, just to exercise the parsing
//             * JSON I guess? OR is our own format better?
//                 * How painful is defining an ASCII map in JSON?
//     * Make the theme changeable, including graphics for now
//     * Will need to figure out how this works for the wasm version. Uploadable file?
//     * Will need to implement for desktop too, even if how it works is a little more clear
// * Fill out the other interactions: ✔
//    * Get told that there is a specific thing by the one that wants it
//    * A proper "You won" screen
//        * Probably make this customizable too
// * Make it more complex by having a locked door that you get the key for by getting one person a thing, that prevents you from getting a second person a thing ✘
//    * Went in a different direction: doors are all portals to other places at the moment, as opposed to walls that can be removed
//        * Could be worth coming back to that kind of door later
// * Add a way to have just collecting a thing unlock a door ✔
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
//            * Ensure the key spawns ✔
//                * Add to config as new item entry
//                    * Turned out to need more than that!
//            * Have the portal start locked ✔
//                * Spawn only the locked door at the start
//                    * Have a "NOT_SPAWNED_AT_START" flag that defaults to false
//                        * Less work than marking all the other entities
//            * Make collecting the key open the corresponding door ✔
//                * Allow it to work for multiple doors in the future
//                    * Markup key as transforming instances of one entity ID into another
//            * Put items in the NPC's pockets, and have them actually give them to you when you give them a thing ✔
//            * Once it has been shown to work for one door, actually make multiple doors, where only one key works on each ✔
//                * Initially, they can both be victory doors
//            * Actually chain the desires together, so one NPC has what the other wants, and the other has the key, and the first item is on the ground ✔
//                * pull in mini_kanren for this, and see if that works out
//                    * Verdict: Too many weird, unclear-how-to-debug issues to make that a good idea.
//                * mini_kanren output was  in the form of a list of item, astract-location pairs.
//                  Like `[("floor", "item 1"), ("npc 1", "item 2"), ("npc 2", "goal")]`
//                  I think a list of structs like that, which will likely have a segment number later, makes sense as
//                  an output format. The Constraints struct close but not exactly a list of those already.


// Steps for "Add hallways between rooms that we'll figure out a way to make more interesting later"
// * Define a second room. Have going through a door take you there
//    * Make that door always open for now
// * Spawn a return door in that room
//    * Spawn next to it when entering
// * Allow entities to spawn in either room, relying on the door being always open to make things solvable
// * firgure out how to handle the door
// * Add a hallway between the two rooms, which doesn't need to participate in the puzzle at all.
//    * For now, every hallway can be the same, short of like one tile between doors or whatever
// * Define more NPCs and items, confirm that larger puzzles still work
// * Add more possible rooms, and connect them with open hallways for now
//    * if it helps, can have them all spoke off the first room to start with.
//      But once that's working, the next step is allowing arbitrary connections between rooms.
// * Allow hallways between the rooms to be locked by keys. Do some checking to confirm that seeds are solvable, if that seems in doubt


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
        COLLECTABLE = models::COLLECTABLE,
        STEPPABLE = models::STEPPABLE,
        VICTORY = models::VICTORY,
        DOOR = models::DOOR,
        NOT_SPAWNED_AT_START = 1 << 4,
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
pub use config::{Config, EntityDef, EntityDefFlags, TileFlags, COLLECTABLE, STEPPABLE, VICTORY, NOT_SPAWNED_AT_START, DOOR};

pub fn to_entity(
    def: &MiniEntityDef,
    x: X,
    y: Y
) -> Entity {
    Entity::new(
        x,
        y,
        def.tile_sprite,
        def.id,
        def.wants.iter().map(|&def_id| models::Desire::new(def_id)).collect(),
        def.on_collect.clone(),
        // This relies on the entity flags being a subset of the entity def flags.
        def.flags,
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
    use models::Location;
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
        segment_id: models::SegmentId,
        needle_flags: TileFlags,
        filter_out: &[Location],
    ) -> Option<Location> {
        // TODO? Cap tiles length or accept this giving a messed up probabilty for large segments?
        let len = segment.tiles.len();
        let offset = xs::range(rng, 0..len as u32) as usize;
        for index in 0..len {
            let i = (index + offset) % len;

            let current_tile_flags = &segment.tiles[i];

            if current_tile_flags & needle_flags == needle_flags {
                let current_xy = i_to_xy(segment.width, i);
                let current_loc = Location{ xy: current_xy, id: segment_id };

                if !filter_out.iter()
                    .any(|&loc| current_loc == loc) {
                    return Some(current_loc);
                }
            }
        }

        None
    }

    pub fn tile_matching_flags(
        rng: &mut Xs,
        segment: &config::WorldSegment,
        segment_id: models::SegmentId,
        needle_flags: TileFlags
    ) -> Option<Location> {
        tile_matching_flags_besides(
            rng,
            segment,
            segment_id,
            needle_flags,
            &[],
        )
    }
}

mod entities {
    use models::{Entity, X, Y, XY, SegmentId};

    use std::collections::{BTreeMap};

    pub type Key = models::Location;

    pub fn entity_key(id: SegmentId, x: X, y: Y) -> Key {
        Key {
            id,
            xy: XY{x, y}
        }
    }

    // Reminder, we're going with Fat Struct for Entities on this project. So, if you are here looking to
    // add an data structure besides Entities that uses entity keys, say some kind of ByEntityKey<A>,
    // then instead consider just adding a field to Entity. If we actually run into perf issues due to
    // entities being large, then we'll know better then than now how to deal with them.

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

        pub fn insert(&mut self, id: SegmentId, entity: Entity) {
            self.map.insert(entity_key(id, entity.x, entity.y), entity);
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
    pub segments: Vec1<WorldSegment>,
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

    pub fn player_key(&self) -> entities::Key {
        entity_key(
            self.segment_id,
            self.player.x,
            self.player.y,
        )
    }

    pub fn get_entity(&self, key: entities::Key) -> Option<&Entity> {
        if key == self.player_key() {
            return Some(&self.player)
        }

        self.mobs.get(key).or_else(|| self.steppables.get(key))
    }

    pub fn get_entity_mut(&mut self, key: entities::Key) -> Option<&mut Entity> {
        if key == self.player_key() {
            return Some(&mut self.player)
        }

        self.mobs.get_mut(key).or_else(|| self.steppables.get_mut(key))
    }

    pub fn warp_player_to(&mut self, target: &DoorTarget) {
        self.segment_id = target.id;
        self.player.x = target.xy.x;
        self.player.y = target.xy.y;

        self.player.offset_x = 0.;
        self.player.offset_y = 0.;
    }
}

fn can_walk_onto(world: &World, id: SegmentId, x: X, y: Y) -> bool {
    let Some(segment) = world.segments.get(id as usize) else {
        return false;
    };
    let Ok(i) = xy_to_i(segment, x, y) else {
        return false;
    };

    if let Some(tile) = segment.tiles.get(i) {
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
    DoorTo(DoorTarget, DoorAnimation),
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

type FrameCount = u16;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DoorAnimation {
    frame: FrameCount,
    is_dramatic: bool,
}

impl DoorAnimation {
    pub fn advance_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub fn is_done(&self) -> bool {
        self.frame >= self.multiple() * 3
    }

    // TODO: reference the config file somehow to determine this.
    //     Probably copy the frames somewhere, instead of referencing it every frame
    pub fn sprite(&self) -> models::TileSprite {
        let multiple = self.multiple();

        match self.frame {
            x if x < multiple => models::DOOR_ANIMATION_FRAME_1,
            x if x < multiple * 2 => models::DOOR_ANIMATION_FRAME_2,
            _ => models::DOOR_ANIMATION_FRAME_3,
        }
    }

    fn multiple(&self) -> FrameCount {
        if self.is_dramatic { 50 } else { 10 }
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
    pub goal_door_tile_sprite: TileSprite,
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
    NoGoalItemFound,
    CouldNotPlaceItem(MiniEntityDef),
    InvalidDesireID(MiniEntityDef, SegmentId),
    NonItemWasDesired(MiniEntityDef, MiniEntityDef, SegmentId),
    InvalidSpeeches(speeches::PushError),
    InvalidInventoryDescriptions(speeches::PushError),
    // TODO? Push this back into the config, with a limited length Vec type?
    TooManySegments,
    ZeroSegments,
}

impl State {
    pub fn new(seed: Seed, config: Config) -> Result<State, Error> {
        use config::{FLOOR, DOOR_START, PLAYER_START, NPC_START, ITEM_START};

        let mut rng = xs::from_seed(seed);

        let mut segments = Vec::with_capacity(16);
        let mut config_segments = Vec::with_capacity(16);

        // TODO randomize the amount here
        for _ in 0..4 {
            // TODO? Cap the number of segments, or just be okay with the first room never being in the 5 billions, etc?
            let index = xs::range(&mut rng, 0..config.segments.len() as u32) as usize;
    
            let config_segment = &config.segments[index];
    
            let tiles: Vec<_> = config_segment.tiles.iter().map(
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

            segments.push(
                WorldSegment {
                    width: config_segment.width,
                    tiles,
                },
            );
            config_segments.push(
                config_segment
            );
        }

        let Ok(segments) = Vec1::try_from(segments) else {
            return Err(Error::ZeroSegments);
        };

        let Ok(config_segments) = Vec1::try_from(config_segments) else {
            return Err(Error::ZeroSegments);
        };

        let Ok(segments_count) = SegmentId::try_from(segments.len()) else {
            return Err(Error::TooManySegments);
        };

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
            } else if def.flags & DOOR == DOOR {
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

        let player = Entity {
            sprite: models::PLAYER_SPRITE,
            ..<_>::default()
        };

        let mut world = World {
            player,
            segment_id: <_>::default(),
            segments,
            steppables: <_>::default(),
            mobs: <_>::default(),
        };

        let first_segment = world.segments.first();
        let first_config_segment = config_segments.first();
        let first_segment_id = 0;

        let mut placed_already = Vec::with_capacity(16);

        let p_loc = random::tile_matching_flags(&mut rng, first_config_segment, first_segment_id, PLAYER_START)
            .or_else(|| 
                random::passable_tile(&mut rng, first_segment)
                    .map(|xy| Location { xy, id: first_segment_id, })
            )
            .ok_or(Error::CannotPlacePlayer)?;
        world.player.x = p_loc.xy.x;
        world.player.y = p_loc.xy.y;
        placed_already.push(p_loc);

        for door_def in &door_defs {
            if door_def.flags & NOT_SPAWNED_AT_START == NOT_SPAWNED_AT_START { continue }
            if door_def.flags & STEPPABLE != STEPPABLE { continue }

            for i in 0..segments_count {
                for j in i..segments_count {
                    if i == j { continue }

                    let config_segment_1 = &config_segments[i as usize];
                    let config_segment_2 = &config_segments[j as usize];

                    let d_1_loc = random::tile_matching_flags_besides(
                        &mut rng,
                        config_segment_1,
                        i,
                        FLOOR | DOOR_START,
                        &placed_already,
                    ).ok_or(Error::CannotPlaceDoor)?;
                    placed_already.push(d_1_loc);

                    let d_2_loc = random::tile_matching_flags_besides(
                        &mut rng,
                        config_segment_2,
                        j,
                        FLOOR | DOOR_START,
                        &placed_already,
                    ).ok_or(Error::CannotPlaceDoor)?;
                    placed_already.push(d_2_loc);
        
                    let mut door_1 = to_entity(
                        door_def,
                        d_1_loc.xy.x,
                        d_1_loc.xy.y
                    );
                    door_1.door_target = d_2_loc;

                    world.steppables.insert(i, door_1);

                    let mut door_2 = to_entity(
                        door_def,
                        d_2_loc.xy.x,
                        d_2_loc.xy.y
                    );

                    door_2.door_target = d_1_loc;

                    world.steppables.insert(j, door_2);
                }
            }
        }

        let mut goal_info = None;

        // TODO? mark up the goal items in the config?
        // TODO? Helper for this pattern of "find a random place to start iterating?"
        let index_offset = xs::range(&mut rng, 0..item_defs.len() as u32) as usize;
        'find_goal: for iteration_index in 0..item_defs.len() {
            let index = (iteration_index + index_offset) % item_defs.len();

            let item_def = &item_defs[index];

            if item_def.on_collect.is_empty() { continue }

            for action in &item_def.on_collect {
                match action {
                    CollectAction::Transform{ from, to: _ } => {
                        let Some(door_def) = door_defs.iter().find(|d| d.id == *from) else {
                            continue
                        };

                        if door_def.flags & NOT_SPAWNED_AT_START == NOT_SPAWNED_AT_START {
                            continue
                        }

                        let d_loc = random::tile_matching_flags_besides(
                            &mut rng,
                            first_config_segment,
                            first_segment_id,
                            FLOOR | DOOR_START,
                            &placed_already,
                        ).ok_or(Error::CannotPlaceDoor)?;

                        world.steppables.insert(
                            first_segment_id,
                            to_entity(
                                door_def,
                                d_loc.xy.x,
                                d_loc.xy.y
                            ),
                        );

                        placed_already.push(d_loc);

                        let goal_door_tile_sprite = door_def.tile_sprite;

                        goal_info = Some((
                            item_def.into(),
                            goal_door_tile_sprite,
                        ));

                        break 'find_goal
                    }
                }
            }
        }

        let Some((goal_item_def, goal_door_tile_sprite)) = goal_info else {
            return Err(Error::NoGoalItemFound);
        };

        #[derive(Debug)]
        enum AbstractLocation<'defs> {
            Floor,
            NpcPocket(&'defs MiniEntityDef),
        }

        #[derive(Debug)]
        struct ItemSpec<'defs> {
            // segment_id: SegmentId, // Expected to be added later
            item_def: &'defs MiniEntityDef,
            location: AbstractLocation<'defs>,
        }

        #[derive(Debug)]
        struct Constraints<'defs> {
            item_specs: Vec<ItemSpec<'defs>>,
        }

        fn select_constraints<'defs>(
            rng: &mut Xs,
            all_desires: &[config::DesireRef<'defs>],
            goal_item_def: &'defs MiniEntityDef,
        ) -> Constraints<'defs> {
            let desire_count_to_use = xs::range(rng, 1..all_desires.len() as u32 + 1) as usize;
            // We end up with 1 more spec than the desires we use, becauase we start with one that doesn't use any.
            let target_len = desire_count_to_use + 1;
            let initial_index = xs::range(rng, 0..all_desires.len() as u32) as usize;

            let mut item_specs: Vec<_> = Vec::with_capacity(target_len);

            let mut tries = 0;
            while item_specs.len() < target_len && tries < 16 {
                tries += 1;
                item_specs.clear();

                item_specs.push(ItemSpec{
                    item_def: goal_item_def,
                    location: AbstractLocation::Floor,
                });

                let mut index = initial_index;
                while item_specs.len() < target_len {
                    // Select the index or not, at a rate proportional to how many we need.
                    if (xs::range(rng, 0..all_desires.len() as u32 + 1) as usize) < target_len {
                        let Some(last) = item_specs.pop() else {
                            debug_assert!(false, "item_specs.pop() == None");
                            continue
                        };

                        let desire = all_desires[index];

                        item_specs.push(ItemSpec{
                            item_def: desire.item_def,
                            location: last.location,
                        });

                        item_specs.push(ItemSpec{
                            item_def: last.item_def,
                            location: AbstractLocation::NpcPocket(desire.mob_def),
                        });
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
            }

            Constraints {
                item_specs,
            }
        }

        let constraints: Constraints = select_constraints(&mut rng, &all_desires, goal_item_def);

        for item_spec in constraints.item_specs {
            let mut attempts = 0;

            while attempts < 16 {
                attempts += 1;

                use AbstractLocation::*;
                match item_spec.location {
                    Floor => {
                        if let Some(item_loc) = random::tile_matching_flags_besides(
                            &mut rng,
                            first_config_segment,
                            first_segment_id,
                            ITEM_START,
                            &placed_already,
                        ) {
                            world.steppables.insert(
                                first_segment_id,
                                to_entity(
                                    item_spec.item_def,
                                    item_loc.xy.x,
                                    item_loc.xy.y
                                ),
                            );
                            placed_already.push(item_loc);
                            break
                        }
                    },
                    NpcPocket(npc_def) => {
                        if let Some(npc_loc) = random::tile_matching_flags_besides(
                            &mut rng,
                            first_config_segment,
                            first_segment_id,
                            NPC_START,
                            &placed_already,
                        ) {
                            placed_already.push(npc_loc);

                            let mut mob = to_entity(
                                npc_def,
                                npc_loc.xy.x,
                                npc_loc.xy.y
                            );

                            mob.inventory.push(
                                to_entity(
                                    item_spec.item_def,
                                    npc_loc.xy.x,
                                    npc_loc.xy.y
                                )
                            );

                            world.mobs.insert(
                                first_segment_id,
                                mob,
                            );

                            break
                        }
                    }
                }
            }

            if attempts >= 16 {
                return Err(Error::CouldNotPlaceItem(item_spec.item_def.to_owned()));
            }
        }

        Ok(State {
            rng,
            world,
            mode: <_>::default(),
            fade_message_specs: <_>::default(),
            shake_amount: <_>::default(),
            speeches,
            inventory_descriptions,
            entity_defs,
            goal_door_tile_sprite,
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
            if let Some(steppable) = self.world.steppables.get(
                entity_key(self.world.segment_id, new_x, new_y)
            ) {
                if steppable.flags & STEPPABLE != STEPPABLE {
                    // Doors or other things which may become steppable, but aren't now.
                    // TODO: Is the world.steppables/world.mobs distinction worth it?
                    return
                }
            }

            // TODO? Worth making every update to any entities x/y update the offset?
            self.world.player.offset_x = offset::X::from(self.world.player.x) - offset::X::from(new_x);
            self.world.player.offset_y = offset::Y::from(self.world.player.y) - offset::Y::from(new_y);

            self.world.player.x = new_x;
            self.world.player.y = new_y;

            let key = self.world.player_key();

            if let Some(steppable) = self.world.steppables.get(key) {
                if steppable.is_collectable() {
                    self.shake_amount = 5;

                    let steppable = self.world.steppables.remove(key)
                        // Yes, this relies on game updates being on a single thread, 
                        // but we'd presumbably need to change a bunch of other things
                        // too, to make multiple threads work.
                        .expect("We just checked for it a moment ago!");
                    self.push_inventory(key, steppable);
                } else if steppable.is_victory() {
                    let mut animation = DoorAnimation::default();
                    animation.is_dramatic = true;
                    self.mode = Mode::Victory(animation);
                } else if steppable.is_door() {
                    self.mode = Mode::DoorTo(steppable.door_target, <_>::default());
                } else {
                    // Effectively just scenery.
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

        let Some(interactable) = self.world.mobs.get_mut(key).or_else(|| self.world.steppables.get_mut(key)) else {
            self.fade_message_specs.push(
                FadeMessageSpec::new(format!("there's nobody there."), entity.xy())
            );
            return
        };

        let mut post_action = PostTalkingAction::NoOp;
        for desire in &mut interactable.desires {
            use models::DesireState::*;
            // Check if interactable should notice the player's item.
            if desire.state == Unsatisfied
            && entity.inventory.iter().any(|e| e.def_id == desire.def_id) {
                desire.state = SatisfactionInSight;
                post_action = PostTalkingAction::TakeItem(key, desire.def_id);
            }
        }

        let speeches_key = interactable.speeches_key();

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
                    models::entity_article_phrase(interactable),
                ),
                entity.xy()
            )
        );
    }

    pub fn tick(&mut self) {
        macro_rules! advance_door_animation {
            ($animation: expr) => ({
                let player = &self.world.player;

                // Wait until the player animating towrads a door has settled first.
                if player.offset_x == 0. && player.offset_y == 0. {
                    $animation.advance_frame();
                }
            })
        }

        match &mut self.mode {
            Mode::DoorTo(target, animation) => {
                advance_door_animation!(animation);

                if animation.is_done() {
                    self.world.warp_player_to(target);
                }
            }
            Mode::Victory(animation) => {
                advance_door_animation!(animation);

                /* fall through to rest of method */
            },
            Mode::Walking => { /* fall through to rest of method */ }
            Mode::Inventory { .. }
            | Mode::Talking(..) => return,
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

    pub fn push_inventory(&mut self, target_key: entities::Key, item: Entity) {
        if target_key == self.world.player_key() {
            for action in &item.on_collect {
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
        }

        if let Some(target) = self.world.get_entity_mut(target_key) {
            target.inventory.push(item);
        }
    }
}