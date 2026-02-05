use features::invariant_assert;
use models::{
    config::{Config},
    offset,
    sprite,
    speeches,
    CollectAction,
    DefId,
    Entity,
    Location,
    MiniEntityDef,
    Speech,
    Speeches,
    TileSprite,
    XY,
    ShakeAmount,
    is_passable,
    xy_to_i,
};
type DoorTarget = Location;

use xs::{Xs, Seed};

use platform_types::{arrow_timer::{ArrowTimer}, Dir};
use vec1::Vec1;
use world::{World, HallwayStates};
pub use world::{EntityKey};
pub use world::hallway::State as HallwayState;

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
//                            * Well after implementing a bunch of things in Rhai, when tracking down another bug, we discovered that Rhai has UB that can be detected by Miri.
//                              * A dep has UB, and they are reasonably reluctant to change it, since it is exposed in the API: https://github.com/rhaiscript/rhai/issues/816
//                              * Also apparently a use after-free: https://github.com/rhaiscript/rhai/issues/894
//                              * So I think it maybe makes sense to look at other alternatives again. Looking back at that survey, Rune seems the most interesting. 
//                                This is because many of the other options either seem not actively supported, or they are bash-like or lisp-like and those kinds of languages make 
//                                working with product types like structs either complciated or impossible. Dyon is another potentially viable option, but it lists a bunch of "new things" 
//                                and that seems like it might make it weirder than makes sense for a language users will be icking up not for its own sake.
//                                Rune also has WASM support. https://github.com/rune-rs/rune/issues/662
//
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
// * Define a second room. Have going through a door take you there ✔
//    * Make that door always open for now
// * Spawn a return door in that room ✔
//    * Spawn next to it when entering
//        * On top is good enough, and so far doesn't cause issues like sending you back. We can change later if needed.
// * Allow entities to spawn in either room, relying on the door being always open to make things solvable
// * Figure out how to handle the doors with keys, in the generation
//    * I think figuring out the rooms first, then door and key placements next, then deciding on NPCs should work.
//    * Oh, no wait! Cyclic generation like from Unexplored! Yes, let's do that!
//        * Generation Steps:
//            * Produce a random undirected graph of the right size, where each node has at least two edges to different nodes.
//            * Make paritiions of the graph into spheres, where they are blocked from each other by locks. Assign keys to each lock
//            * Place the key for each lock within the inner sphere where the way out is locked by it
//                * Can sort the rooms in a sphere by distance and pick ones further in, but not to far, say 70%
//                * Steps here would be pick a spot for the npc to be, then pick another spot in the sphere for their desire to be
//                    Can make the key tending to be far or near to the npc a parameter
//        * Implementation steps:
//            * More or less backwards. That is, start with the usage code, and figure out what the spheres and graph structure should
//              be like, based on how it's accessed. Then work backwards from there.
// * Add a hallway between the two rooms, which doesn't need to participate in the puzzle at all.
//    * For now, every hallway can be the same, short of like one tile between doors or whatever
//        * A null hallway that just jumps you there should be an option too
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
//    * Disgaea's geo-block puzzles
//    * Boulder Dash maybe?

fn warp_player_to(world: &mut World, target: &DoorTarget) {
    world.segment_id = target.segment_id;
    world.player.xy = target.xy;

    world.player.offset = offset::XY::ZERO;
}

fn can_walk_onto(world: &World, key @ EntityKey { segment_id, xy: XY{ x, y } }: EntityKey) -> bool {
    let Some(segment) = world.segments.get(usize::from(segment_id)) else {
        return false;
    };
    let Ok(i) = xy_to_i(segment, x, y) else {
        return false;
    };

    if let Some(tile) = segment.tiles.get(i) {
        if let Some(mob) = world.mobs.get(key) {
            if !mob.is_steppable() {
                return false;
            }
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
    TakeItem(EntityKey, DefId),
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
    Hallway{
        source: Location,
        target: Location,
    },
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
    pub hallway_states: HallwayStates,
}

impl State {
    pub fn all_entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.world.all_entities_mut()
    }
}

pub type Error = world::Error;

impl State {
    pub fn new(specs: &sprite::Specs, seed: Seed, config: Config) -> Result<State, Error> {
        let mut rng = xs::from_seed(seed);

        let world::Generated {
            world,
            speeches,
            inventory_descriptions,
            entity_defs,
            goal_door_tile_sprite,
            hallway_states,
        } = world::generate(&mut rng, &config, specs)?;

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
            hallway_states,
        })
    }
}

fn xy_in_dir(XY { x, y }: XY, dir: Dir) -> Option<XY> {
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
        let Some(new_xy) = xy_in_dir(self.world.player.xy, dir) else {
            return
        };

        let new_key = self.world.local_key(new_xy);

        if can_walk_onto(&self.world, new_key) {
            if let Some(mob) = self.world.mobs.get(new_key) {
                if !mob.is_steppable() {
                    // Doors or other things which may become steppable, but aren't now.
                    return
                }
            }

            // TODO? Worth making every update to any entities x/y update the offset?
            self.world.player.offset = offset::XY::from(self.world.player.xy) - offset::XY::from(new_xy);

            self.world.player.xy = new_xy;

            let key = self.world.player_key();

            if let Some(mob) = self.world.mobs.get(key) {
                if mob.is_collectable() {
                    self.shake_amount = 5;

                    let steppable = self.world.mobs.remove(key)
                        // Yes, this relies on game updates being on a single thread,
                        // but we'd presumbably need to change a bunch of other things
                        // too, to make multiple threads work.
                        .expect("We just checked for it a moment ago!");
                    self.push_inventory(key, steppable);
                } else if mob.is_victory() {
                    let mut animation = DoorAnimation::default();
                    animation.is_dramatic = true;
                    self.mode = Mode::Victory(animation);
                } else if mob.is_door() {
                    self.mode = Mode::DoorTo(mob.door_target, <_>::default());
                } else {
                    // Effectively just scenery.
                }
            }
        }
    }

    #[must_use]
    pub fn interact(&mut self, dir: Dir) {
        let entity = &self.world.player;

        let Some(target_xy) = xy_in_dir(entity.xy, dir) else {
            self.fade_message_specs.push(FadeMessageSpec::new(format!("there's nothing there."), entity.xy));
            return
        };

        let key = self.world.local_key(target_xy);

        let entity = &mut self.world.player;

        let Some(interactable) = self.world.mobs.get_mut(key) else {
            self.fade_message_specs.push(
                FadeMessageSpec::new(format!("there's nobody there."), entity.xy)
            );
            return
        };

        let mut post_action = PostTalkingAction::NoOp;
        for desire in &mut interactable.transformable.wants {
            use models::DesireState::*;
            // Check if interactable should notice the player's item.
            if desire.state == Unsatisfied
            && entity.inventory.iter().any(|e| e.transformable.id == desire.def_id) {
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
                entity.xy
            )
        );
    }

    pub fn tick(&mut self) {
        macro_rules! advance_door_animation {
            ($animation: expr) => ({
                let player = &self.world.player;

                // Wait until the player animating towrads a door has settled first.
                if player.offset == offset::XY::ZERO {
                    $animation.advance_frame();
                }
            })
        }

        match &mut self.mode {
            Mode::DoorTo(target, animation) => {
                if animation.is_done() {
                    let target = *target;
                    let source = self.world.player_key();
                    // Skip a frame of waiting by checking early.
                    // This shouldn't be relied on for correctness, since there's a TOCTOU issue.
                    let do_warp = if let Some(hallway) = self.hallway_states.get_mut(source, target) {
                        hallway.is_complete()
                    } else {
                        true
                    }; 

                    if do_warp {
                        warp_player_to(&mut self.world, &target);
                        self.mode = Mode::Walking;
                    } else {
                        self.mode = Mode::Hallway{ source, target };
                    }
                } else {
                    // Do this last, so that the last frame is shown
                    advance_door_animation!(animation);
                }
            }
            Mode::Hallway{ source, target } => {
                let do_warp = if let Some(hallway) = self.hallway_states.get_mut(*source, *target) {
                    hallway.tick();
        
                    hallway.is_complete()
                } else {
                    true
                };
                if do_warp {
                    warp_player_to(&mut self.world, target);
                    self.mode = Mode::Walking;
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
            entity.offset.decay();
        }
    }

    pub fn push_inventory(&mut self, target_key: EntityKey, item: Entity) {
        if target_key == self.world.player_key() {
            for action in &item.transformable.on_collect {
                match action {
                    CollectAction::Transform(models::Transform{ from, to }) => {
                        if let Some(to_def) = self.entity_defs.get((*to) as usize) {
                            world::transform_all_matching(&mut self.world, *from, to_def);
                        } else {
                            invariant_assert!(false, "Why are we trying to transform something into something that doesn't exist? to {to}");
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