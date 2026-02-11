use features::{invariant_assert};
use models::{
    config::{Config},
    consts::{ITEM_START, NPC_START, PLAYER_START, COLLECTABLE, STEPPABLE, VICTORY, NOT_SPAWNED_AT_START, DOOR, FLOOR, DOOR_START},
    speeches,
    sprite,
    CollectAction, DefId, Entity, EntityTransformable, Location, MiniEntityDef, Specs, Speech, Speeches, Tile, TileSprite, Transform, WorldSegment, XY, SegmentId
};
use vec1::Vec1;
use xs::{Xs};

pub const TILES_PER_ROW: TileSprite = 6;
pub const WALL_SPRITE: TileSprite = 0;
pub const FLOOR_SPRITE: TileSprite = 1;
pub const PLAYER_SPRITE: TileSprite = 2;
pub const DOOR_ANIMATION_FRAME_1: TileSprite = 9;
pub const DOOR_ANIMATION_FRAME_2: TileSprite = DOOR_ANIMATION_FRAME_1 + TILES_PER_ROW;
pub const DOOR_ANIMATION_FRAME_3: TileSprite = DOOR_ANIMATION_FRAME_2 + TILES_PER_ROW;

pub fn is_passable(tile: &Tile) -> bool {
    tile.sprite == FLOOR_SPRITE
}

/// Returns a phrase like "a thing" or "an entity".
pub fn entity_article_phrase(entity: &Entity) -> &str {
    match entity.transformable.tile_sprite {
        WALL_SPRITE => "a wall",
        FLOOR_SPRITE => "a floor",
        PLAYER_SPRITE => "a me(?!)",
        _ => "a whatever-this-is",
    }
}

mod entities {
    use models::{Entity, XY, SegmentId};

    use std::collections::{BTreeMap};

    pub type Key = models::Location;

    pub fn entity_key(segment_id: SegmentId, xy: XY) -> Key {
        Key {
            segment_id,
            xy,
        }
    }

    pub fn key_for_entity(segment_id: SegmentId, entity: &Entity) -> Key {
        entity_key(segment_id, entity.xy)
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
            self.map.range(entity_key(id, XY::MIN)..=entity_key(id, XY::MAX))
        }

        pub fn insert(&mut self, id: SegmentId, entity: Entity) {
            self.map.insert(entity_key(id, entity.xy), entity);
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
use entities::{Entities, entity_key, key_for_entity};

pub type EntityKey = entities::Key;

pub mod hallway {
    use features::{invariant_assert};
    use crate::entities::Key as EntityKey;

    use std::collections::BTreeMap;

    #[derive(Clone, Debug)]
    pub enum State {
        IcePuzzle(ice_puzzle::State),
        // Staff Whacking Ordeal Required, Duh
        SWORD(sword::State),
    }

    impl State {
        pub fn is_complete(&self) -> bool {
            use State::*;
            match self {
                IcePuzzle(inner) => inner.is_complete(),
                SWORD(inner) => inner.is_complete(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Key {
        lower: EntityKey,
        higher: EntityKey,
    }

    impl Key {
        fn new(a: EntityKey, b: EntityKey) -> Self {
            let lower = core::cmp::min(a, b);
            let higher = core::cmp::max(a, b);

            invariant_assert!(
                (lower != higher) // The two part should be distinct
                || (a == b),       // unless they were the same in the first place
                "Lower and higher hallway key parts were the same!"
            );
            // A separate assert in case we decide there's a reason for allowing this.
            invariant_assert!(a != b, "Attempted to make hallway key from two identical parts. Probably a bug?");

            Self {
                lower,
                higher,
            }
        }
    }

    #[derive(Clone, Default)]
    pub struct States {
        map: BTreeMap<Key, State>,
    }

    impl States {
        pub fn insert(&mut self, a: EntityKey, b: EntityKey, state: State) -> Option<State> {
            self.map.insert(Key::new(a, b), state)
        }

        pub fn get(&self, a: EntityKey, b: EntityKey) -> Option<&State> {
            self.map.get(&Key::new(a, b))
        }

        pub fn get_mut(&mut self, a: EntityKey, b: EntityKey) -> Option<&mut State> {
            self.map.get_mut(&Key::new(a, b))
        }
    }
}
pub use hallway::{States as HallwayStates};

#[derive(Clone, Default)]
pub struct World {
    pub segments: Vec1<WorldSegment>,
    /// The ID of the current segment we are in.
    pub segment_id: SegmentId,
    pub player: Entity,
    pub mobs: Entities,
}

impl World {
    pub fn all_entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        std::iter::once(&mut self.player).chain(self.mobs.all_entities_mut())
    }

    pub fn player_key(&self) -> EntityKey {
        entity_key(
            self.segment_id,
            self.player.xy,
        )
    }

    pub fn local_key(&self, xy: XY) -> EntityKey {
        entity_key(
            self.segment_id,
            xy,
        )
    }

    pub fn get_entity(&self, key: EntityKey) -> Option<&Entity> {
        if key == self.player_key() {
            return Some(&self.player)
        }

        self.mobs.get(key)
    }

    pub fn get_entity_mut(&mut self, key: EntityKey) -> Option<&mut Entity> {
        if key == self.player_key() {
            return Some(&mut self.player)
        }

        self.mobs.get_mut(key)
    }
}

pub fn to_entity(
    def: &MiniEntityDef,
    xy: XY,
) -> Entity {
    Entity::new(
        xy,
        def.into(),
    )
}

fn transform_entity(entity: &mut Entity, def: &MiniEntityDef) {
    // TODO? Is it worth storing pre-processed entity Defs, instead of the whole thing? In terms of any of
    //       reduced memory usage, less work needed to do these transforms, or reduced mixing of concerns?
    // We intentionally keep everything we dont want to be affected by this function in different fields.
    // This decision makes maintaining this function a non-issue, where it would othersie be a pain each
    // time we add fields anywhere.
    entity.transformable = def.into();
}

pub fn transform_all_matching(world: &mut World, from_def_id: DefId, to_def: &MiniEntityDef) {
    for entity in world.all_entities_mut() {
        if entity.def_id() == from_def_id {
            transform_entity(entity, to_def);
        }
    }
}

mod random {
    use models::{
        config, consts::{TileFlags}, Location,
        XY,
        i_to_xy,
        WorldSegment,
    };
    use xs::{Xs};

    pub fn passable_tile(rng: &mut Xs, segment: &WorldSegment) -> Option<XY> {
        // TODO? Cap tiles length or accept this giving a messed up probabilty for large segments?
        let len = segment.tiles.len();
        let offset = xs::index(rng, 0..len);
        for index in 0..len {
            let i = (index + offset) % len;

            let tile = &segment.tiles[i];

            if crate::is_passable(tile) {
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
        let offset = xs::index(rng, 0..len);
        for index in 0..len {
            let i = (index + offset) % len;

            let current_tile_flags = &segment.tiles[i];

            if current_tile_flags & needle_flags == needle_flags {
                let current_xy = i_to_xy(segment.width, i);
                let current_loc = Location{ xy: current_xy, segment_id };

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

#[derive(Clone, Copy, Debug)]
pub struct DesireRef<'defs> {
    pub mob_def: &'defs MiniEntityDef,
    pub item_def: &'defs MiniEntityDef,
}

#[derive(Debug)]
pub enum Error {
    CannotPlacePlayer,
    CannotPlaceDoor,
    NoEntityDefs,
    NoMobsFound,
    NoItemsFound,
    NoDoorsFound,
    NoOpenDoorFound,
    NoGoalItemFound,
    NotEnoughNonFinalLockAndKeysFound,
    CouldNotPlaceItem{ def: MiniEntityDef, config_index: usize },
    InvalidDesireID(MiniEntityDef, SegmentId),
    NonItemWasDesired(MiniEntityDef, MiniEntityDef, SegmentId),
    InvalidSpeeches(speeches::PushError),
    InvalidInventoryDescriptions(speeches::PushError),
    // TODO? Push this back into the config, with a limited length Vec type?
    TooManySegments,
    ZeroSegments,
}

#[derive(Debug, PartialEq, Eq)]
struct ItemAndDoorDefs<'defs> {
    item_defs: &'defs [MiniEntityDef],
    door_defs: &'defs [MiniEntityDef],
}

#[derive(Debug, PartialEq, Eq)]
struct LockAndKey<'defs> {
    lock: &'defs MiniEntityDef,
    key: &'defs MiniEntityDef,
}

fn get_non_final_lock_and_keys<'defs>(
    ItemAndDoorDefs{
        item_defs,
        door_defs,
    }: ItemAndDoorDefs<'defs>,
) -> Option<Vec1<LockAndKey<'defs>>> {
    let mut non_final_lock_and_keys = Vec::with_capacity(16);

    for i in 0..item_defs.len() {
        let item_def = &item_defs[i];

        if item_def.on_collect.is_empty() { continue }

        for action in &item_def.on_collect {
            match action {
                CollectAction::Transform(Transform{ from, to }) => {
                    let Some(open_door_def) = door_defs.iter().find(|d| d.id == *to) else {
                        continue
                    };

                    // We are looking for all the non-victory doors.
                    if open_door_def.flags & VICTORY == VICTORY {
                        continue
                    }

                    let Some(locked_door_def) = door_defs.iter().find(|d| d.id == *from) else {
                        continue
                    };

                    // We are planning to spawn these at the start
                    if locked_door_def.flags & NOT_SPAWNED_AT_START == NOT_SPAWNED_AT_START { continue }
                    // We want locked doors
                    if locked_door_def.flags & STEPPABLE == STEPPABLE { continue }

                    non_final_lock_and_keys.push(
                        LockAndKey {
                            lock: locked_door_def,
                            key: item_def,
                        }
                    );

                    break
                }
            }
        }
    }

    Vec1::try_from(non_final_lock_and_keys).ok()
}

#[cfg(test)]
mod get_non_final_lock_and_keys_works {
    use super::*;
    use models::{CollectAction, MiniEntityDef, Transform};
    use vec1::Vec1;

    const SOME_LOCKED_DOOR: MiniEntityDef = MiniEntityDef {
        id: 0,
        flags: DOOR,
        tile_sprite: 0,
        on_collect: vec![],
        wants: vec![],
    };

    const SOME_OPEN_DOOR: MiniEntityDef = MiniEntityDef {
        id: 1,
        flags: DOOR,
        tile_sprite: 0,
        on_collect: vec![],
        wants: vec![],
    };

    fn some_key() -> MiniEntityDef {
        MiniEntityDef {
            id: 2,
            flags: 0,
            tile_sprite: 0,
            on_collect: vec![CollectAction::Transform(
                Transform{ from: SOME_LOCKED_DOOR.id, to: SOME_OPEN_DOOR.id }
            )],
            wants: vec![],
        }
    }

    const LOCKED_VICTORY_DOOR: MiniEntityDef = MiniEntityDef {
        id: 3,
        flags: DOOR,
        tile_sprite: 0,
        on_collect: vec![],
        wants: vec![],
    };

    const OPEN_VICTORY_DOOR: MiniEntityDef = MiniEntityDef {
        id: 4,
        flags: DOOR | VICTORY | STEPPABLE,
        tile_sprite: 0,
        on_collect: vec![],
        wants: vec![],
    };

    fn victory_key() -> MiniEntityDef {
        MiniEntityDef {
            id: 5,
            flags: 0,
            tile_sprite: 0,
            on_collect: vec![CollectAction::Transform(
                Transform{ from: LOCKED_VICTORY_DOOR.id, to: OPEN_VICTORY_DOOR.id }
            )],
            wants: vec![],
        }
    }

    #[test]
    fn on_the_basics() {
        assert_eq!(
            get_non_final_lock_and_keys(
                ItemAndDoorDefs {
                    item_defs: &[],
                    door_defs: &[],
                },
            ),
            None
        );

        let some_key = some_key();

        assert_eq!(
            get_non_final_lock_and_keys(
                ItemAndDoorDefs {
                    item_defs: &[some_key.clone()],
                    door_defs: &[SOME_LOCKED_DOOR, SOME_OPEN_DOOR],
                },
            ),
            Some(
                Vec1::singleton(
                    LockAndKey {
                        lock: &SOME_LOCKED_DOOR,
                        key: &some_key,
                    }
                )
            )
        );
    }

    #[test]
    fn on_lists_containing_final_lock_and_keys() {
        assert_eq!(
            get_non_final_lock_and_keys(
                ItemAndDoorDefs {
                    item_defs: &[victory_key()],
                    door_defs: &[LOCKED_VICTORY_DOOR],
                },
            ),
            None
        );
    }
}

pub struct Generated {
    pub world: World,
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

pub fn generate(rng: &mut Xs, config: &Config, specs: &sprite::Specs) -> Result<Generated, Error> {
    #[cfg(feature = "invariant-checking")]
    let mut door_target_set = std::collections::HashSet::<DefId>::with_capacity(16);

    macro_rules! track_door_target_set {
        ($door_entity: ident) => {
            #[cfg(feature = "invariant-checking")]
            {
                let door_entity = &$door_entity;

                door_target_set.insert(door_entity.def_id());
            }
        }
    }

    let mut segments = Vec::with_capacity(16);
    let mut config_segments = Vec::with_capacity(16);

    let target_segment_count = xs::range(rng, 4..12);

    for _ in 0..target_segment_count {
        // TODO? Cap the number of segments, or just be okay with the first room never being in the 5 billions, etc?
        let index = xs::index(rng, 0..config.segments.len());

        let config_segment = &config.segments[index];

        let tiles: Vec1<_> = Vec1::map1(
            &config_segment.tiles,
            |tile_flags| {
                Tile {
                    sprite: if tile_flags & FLOOR != 0 {
                        FLOOR_SPRITE
                    } else {
                        WALL_SPRITE
                    },
                }
            }
        );

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

    let mut item_defs = Vec::with_capacity(16);
    let mut door_defs = Vec::with_capacity(16);
    let mut speeches_lists: Vec<models::SpeechesList> = Vec::with_capacity(16);
    let mut inventory_descriptions_lists: Vec<models::SpeechesList> = Vec::with_capacity(16);

    for def in &config.entities {
        // PERF: Is it worth it to avoid this clone?
        speeches_lists.push(def.speeches.clone());
        inventory_descriptions_lists.push(def.inventory_description.clone());
    }

    let speeches = Speeches::try_from(speeches_lists).map_err(Error::InvalidSpeeches)?;
    let inventory_descriptions = Speeches::try_from(inventory_descriptions_lists).map_err(Error::InvalidSpeeches)?;

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
            for &wanted_id in &def.wants {
                if let Some(desired_def) = entity_defs.get(wanted_id.into()) {
                    if desired_def.flags & COLLECTABLE == COLLECTABLE {
                        all_desires.push(DesireRef {
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

    let mut item_defs: Vec1<_> = item_defs.try_into().map_err(|_| Error::NoItemsFound)?;
    let mut door_defs: Vec1<_> = door_defs.try_into().map_err(|_| Error::NoDoorsFound)?;

    xs::shuffle(rng, &mut item_defs);
    xs::shuffle(rng, &mut door_defs);

    let player = Entity {
        transformable: EntityTransformable {
            tile_sprite: PLAYER_SPRITE,
            ..<_>::default()
        },
        ..<_>::default()
    };

    let mut world = World {
        segment_id: <_>::default(),
        segments,
        player,
        mobs: <_>::default(),
    };

    let mut hallway_states = HallwayStates::default();

    macro_rules! assert_door_targets_seem_right {
        () => {
            #[cfg(feature = "invariant-checking")]
            {
                for entity in world.all_entities_mut() {
                    // Just using the mut version because the oter version does not exist as of this writing.
                    let entity = &entity;

                    if entity.is_door() {
                        if entity.is_victory() {
                            invariant_assert!(
                                !door_target_set.contains(&entity.def_id()),
                                "Door target was set on a victory door! That doesn't do anything!\n{entity:#?}"
                            );
                        } else if item_defs.iter().any(|item|
                                item.on_collect.iter()
                                .find(|action|
                                    match action {
                                        CollectAction::Transform(transform) => {
                                            find_def_if_locked_victory_door(&door_defs, *transform)
                                                .map(|def| def.id == entity.def_id())
                                                .unwrap_or(false)
                                        }
                                    }
                                ).is_some()
                            ) {
                            invariant_assert!(
                                !door_target_set.contains(&entity.def_id()),
                                "Door target was set on a non-steppable door that will become a victory door!\n{entity:#?}"
                            );
                        } else {
                            invariant_assert!(
                                door_target_set.contains(&entity.def_id()),
                                "Door target was not set on a door!\n{entity:#?}"
                            );
                        }
                    } else {
                        // If at some point non-doors need door targets, we can relax this. But given that does
                        // not happen, a stronger assertion is better.
                        invariant_assert!(
                            !door_target_set.contains(&entity.def_id()),
                            "Door target was set on non-door!\n{entity:#?}"
                        );
                    }
                }
            }
        }
    }

    let first_segment = world.segments.first();
    let first_config_segment = config_segments.first();
    let first_segment_id = 0;
    invariant_assert!(world.segments.len() <= SegmentId::MAX.into());
    let last_segment_id: SegmentId = (world.segments.len() - 1) as SegmentId;
    let last_config_segment = config_segments.last();

    let mut placed_already = Vec::with_capacity(16);

    let p_loc = random::tile_matching_flags(rng, first_config_segment, first_segment_id, PLAYER_START)
        .or_else(||
            random::passable_tile(rng, first_segment)
                .map(|xy| Location { xy, segment_id: first_segment_id, })
        )
        .ok_or(Error::CannotPlacePlayer)?;
    world.player.xy = p_loc.xy;
    placed_already.push(p_loc);

    let Some(open_door_def) = door_defs.iter().find(|d|
        // A steppable, non-victory door
        d.flags & (VICTORY | STEPPABLE) == STEPPABLE
    ) else {
        return Err(Error::NoOpenDoorFound)
    };

    let mut goal_info = None;

    fn find_def_if_locked_victory_door(
        door_defs: &[MiniEntityDef],
        Transform { from, to }: Transform,
    ) -> Option<&MiniEntityDef> {
        let final_door_def = door_defs.iter().find(|d| d.id == to)?;

        if final_door_def.flags & VICTORY != VICTORY {
            return None
        }

        door_defs.iter().find(|d| d.id == from)
    }

    assert_door_targets_seem_right!();

    // TODO? mark up the goal items in the config?
    // TODO? Helper for this pattern of "find a random place to start iterating?"
    let index_offset = xs::index(rng, 0..item_defs.len());
    'find_goal: for iteration_index in 0..item_defs.len() {
        let index = (iteration_index + index_offset) % item_defs.len();

        let item_def = &item_defs[index];

        if item_def.on_collect.is_empty() { continue }

        for action in &item_def.on_collect {
            match action {
                CollectAction::Transform(transform) => {
                    let Some(initial_door_def) = find_def_if_locked_victory_door(&door_defs, *transform) else {
                        continue
                    };

                    let d_loc = random::tile_matching_flags_besides(
                        rng,
                        last_config_segment,
                        last_segment_id,
                        FLOOR | DOOR_START,
                        &placed_already,
                    ).ok_or(Error::CannotPlaceDoor)?;

                    let door = to_entity(
                        initial_door_def,
                        d_loc.xy,
                    );
                    // We don't need to set targets for victory doors.

                    world.mobs.insert(
                        last_segment_id,
                        door,
                    );

                    placed_already.push(d_loc);

                    let goal_door_tile_sprite = initial_door_def.tile_sprite;

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

    assert_door_targets_seem_right!();

    let Some(non_final_lock_and_keys) = get_non_final_lock_and_keys(
        ItemAndDoorDefs {
            item_defs: &item_defs,
            door_defs: &door_defs,
        },
    ) else {
        return Err(Error::NotEnoughNonFinalLockAndKeysFound);
    };

    #[derive(Debug)]
    struct Sphere<'defs> {
        segment_ids: Vec<SegmentId>,
        goal_item_def: &'defs MiniEntityDef,
    }

    let mut spheres = Vec::with_capacity(16);

    {
        let initial_lak_index = xs::index(rng, 0..non_final_lock_and_keys.len());
        let mut lak_index = initial_lak_index;

        const MIN_PER_SPHERE: u8 = 2;
        const MAX_PER_SPHERE: u8 = 10;

        let mut chunks: Vec<Vec<SegmentId>> = Vec::with_capacity(
            (segments_count / SegmentId::from(MIN_PER_SPHERE)).into()
        );
        chunks.push(Vec::with_capacity(MAX_PER_SPHERE.into()));

        for id in 0..segments_count {
            let final_index = chunks.len() - 1;
            let chunk = &mut chunks[final_index];

            let len = chunk.len();

            if len < usize::from(MIN_PER_SPHERE)
            // This attempts to produce a uniform range
            || xs::range(rng, 0..MAX_PER_SPHERE.into()) == 0 {
                // Add to existing chunk
                chunk.push(id);
            } else {
                // Start new chunk
                let mut new_chunk = Vec::with_capacity(MAX_PER_SPHERE.into());
                new_chunk.push(id);
                chunks.push(new_chunk);
            }
        }

        let chunks_len = chunks.len();
        invariant_assert!(chunks_len >= 2, "We need at least two chunks for the joining to work properly");

        macro_rules! place_door_pair {
            ($door_def: expr, $segment_ids: expr) => {
                assert_door_targets_seem_right!();

                let door_def: &MiniEntityDef = $door_def;
                let (segment_id_i, segment_id_j): (SegmentId, SegmentId) = $segment_ids;

                let config_segment_i = &config_segments[segment_id_i as usize];
                let config_segment_j = &config_segments[segment_id_j as usize];
    
                let d_i_loc = random::tile_matching_flags_besides(
                    rng,
                    config_segment_i,
                    segment_id_i,
                    FLOOR | DOOR_START,
                    &placed_already,
                ).ok_or(Error::CannotPlaceDoor)?;
                placed_already.push(d_i_loc);
    
                let d_j_loc = random::tile_matching_flags_besides(
                    rng,
                    config_segment_j,
                    segment_id_j,
                    FLOOR | DOOR_START,
                    &placed_already,
                ).ok_or(Error::CannotPlaceDoor)?;
                placed_already.push(d_j_loc);
    
                let mut door_i = to_entity(
                    door_def,
                    d_i_loc.xy,
                );
                door_i.door_target = d_j_loc;
                track_door_target_set!(door_i);

                let mut door_j = to_entity(
                    door_def,
                    d_j_loc.xy,
                );
                door_j.door_target = d_i_loc;
                track_door_target_set!(door_j);

                let key_i = key_for_entity(segment_id_i, &door_i);
                let key_j = key_for_entity(segment_id_j, &door_j);
    
                world.mobs.insert(segment_id_i, door_i);
                world.mobs.insert(segment_id_j, door_j);

                assert_door_targets_seem_right!();
    
                invariant_assert!(config.hallways.len() as u128 <= u128::from(u32::MAX));
                let hallway_index = xs::index(rng, 0..config.hallways.len());
                let hallway = &config.hallways[hallway_index];

                use models::config::HallwaySpec;

                match hallway {
                    HallwaySpec::None => {},
                    HallwaySpec::IcePuzzle => {
                        hallway_states.insert(
                            key_i,
                            key_j,
                            hallway::State::IcePuzzle(ice_puzzle::State::new(rng, &specs.ice_puzzles)),
                        );
                    },
                    HallwaySpec::SWORD => {
                        hallway_states.insert(
                            key_i,
                            key_j,
                            hallway::State::SWORD(sword::State::new(rng)),
                        );
                    },
                }

                assert_door_targets_seem_right!();
            }
        }
        for chunk_index in 0..chunks_len {
            let chunk = &chunks[chunk_index];

            // Connect up all the other segments in the chunk with open doors
            for window in chunk
                .windows(2)
                .chain(std::iter::once([chunk[chunk.len() - 1], chunk[0]].as_slice())) {
                assert_eq!(window.len(), 2);

                place_door_pair!(open_door_def, (window[0], window[1]));
            }

            assert_door_targets_seem_right!();
        }

        // From the first to the second last chunk ...
        for chunk_index in 0..(chunks_len - 1) {
            // ... so we can talk about the next chunk
            let next_chunk_index = chunk_index + 1;

            let edge_lak = &non_final_lock_and_keys[lak_index];

            lak_index += 1;
            if lak_index >= non_final_lock_and_keys.len() {
                lak_index = 0;
            }

            if lak_index == initial_lak_index {
                return Err(Error::NotEnoughNonFinalLockAndKeysFound);
            }

            let segment_id = {
                let chunk = &chunks[chunk_index];
                chunk[xs::index(rng, 0..chunk.len())]
            };
            let next_chunk_segment_id = {
                let chunk = &chunks[next_chunk_index];
                chunk[xs::index(rng, 0..chunk.len())]
            };

            place_door_pair!(edge_lak.lock, (segment_id, next_chunk_segment_id));

            spheres.push(Sphere {
                segment_ids: std::mem::take(&mut chunks[chunk_index]),
                goal_item_def: edge_lak.key,
            });
        }

        // Add the last sphere with the goal key and door in it
        spheres.push(Sphere {
            segment_ids: std::mem::take(&mut chunks[chunks_len - 1]),
            goal_item_def,
        });
    }

    assert_door_targets_seem_right!();

    #[cfg(feature = "invariant-checking")]
    {
        // Assert that every segment has at least two doors.

        let mut checklist = vec![0 ;segments_count.into()];

        for segment_id in 0..segments_count {
            for (_, mob) in world.mobs.for_id(segment_id) {
                if mob.is_door() {
                    checklist[usize::from(segment_id)] += 1;
                }
            }
        }

        invariant_assert!(checklist.iter().all(|&x| x > 1), "At least one segment has fewer than two doors! {checklist:?}")
    }

    #[derive(Debug)]
    enum AbstractLocation<'defs> {
        Floor(SegmentId),
        NpcPocket(&'defs MiniEntityDef, SegmentId),
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
        // TODO: Why did we pass this again? Should we still be doing that?
        _world: &World,
        spheres: &[Sphere<'defs>],
        all_desires: &[DesireRef<'defs>],
    ) -> Constraints<'defs> {
        invariant_assert!(_world.segments.len() <= SegmentId::MAX as usize);

        let desire_count_to_use = xs::index(rng, 1..all_desires.len() + 1);
        // We end up with 1 more spec than the desires we use, becauase we start with one that doesn't use any.
        let overall_target_len = desire_count_to_use + 1;

        let average_target_len = core::cmp::max(overall_target_len / spheres.len(), 1);
        let max_sub_target_len = average_target_len * 2;
        invariant_assert!(max_sub_target_len <= u32::MAX as usize); // For random selection later

        let initial_index = xs::index(rng, 0..all_desires.len());

        let mut item_specs: Vec<_> = Vec::with_capacity(overall_target_len);

        let mut tries = 0;
        while item_specs.len() < overall_target_len && tries < 16 {
            tries += 1;
            item_specs.clear();

            for sphere in spheres {
                // TODO: have multiple spheres, delineated by locked doors.
                // Treat each sphere as a separate puzzle, with unlocking
                // the next door as the goal.
                // Keep track of the used desires, so we don't use the same one twice.

                let segment_ids = &sphere.segment_ids;
                invariant_assert!(segment_ids.len() <= u32::MAX as usize);

                macro_rules! random_segment_id {
                    () => {
                        // TODO: We can randomly push some things back into previous spheres, for variety.
                        segment_ids[xs::index(rng, 0..segment_ids.len())]
                    }
                }

                // Start with a solvable puzzle, then add steps, keeping it solvable
                item_specs.push(ItemSpec{
                    item_def: sphere.goal_item_def,
                    location: AbstractLocation::Floor(random_segment_id!()),
                });

                let sub_target_len = xs::index(rng, 1..max_sub_target_len);

                let mut index = xs::index(rng, 0..all_desires.len());

                let initial_spec_len = item_specs.len();

                while item_specs.len() - initial_spec_len < sub_target_len {
                    // Select the index or not, at a rate proportional to how many we need.
                    if (xs::index(rng, 0..all_desires.len() + 1)) < sub_target_len {
                        let Some(last) = item_specs.pop() else {
                            invariant_assert!(false, "item_specs.pop() == None");
                            continue
                        };

                        let desire = all_desires[index];

                        item_specs.push(ItemSpec{
                            item_def: desire.item_def,
                            location: last.location,
                        });

                        item_specs.push(ItemSpec{
                            item_def: last.item_def,
                            location: AbstractLocation::NpcPocket(desire.mob_def, random_segment_id!()),
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
        }

        Constraints {
            item_specs,
        }
    }

    let constraints: Constraints = select_constraints(rng, &world, &spheres, &all_desires);

    for item_spec in constraints.item_specs {
        let mut attempts = 0;

        let mut last_attempted_segment_id = SegmentId::MAX;

        while attempts < 16 {
            attempts += 1;

            use AbstractLocation::*;
            match item_spec.location {
                Floor(segment_id) => {
                    last_attempted_segment_id = segment_id;
                    if let Some(item_loc) = random::tile_matching_flags_besides(
                        rng,
                        &config_segments[usize::from(segment_id)],
                        segment_id,
                        ITEM_START,
                        &placed_already,
                    ) {
                        world.mobs.insert(
                            item_loc.segment_id,
                            to_entity(
                                item_spec.item_def,
                                item_loc.xy,
                            ),
                        );
                        placed_already.push(item_loc);
                        break
                    }
                },
                NpcPocket(npc_def, segment_id) => {
                    last_attempted_segment_id = segment_id;
                    if let Some(npc_loc) = random::tile_matching_flags_besides(
                        rng,
                        &config_segments[usize::from(segment_id)],
                        segment_id,
                        NPC_START,
                        &placed_already,
                    ) {
                        placed_already.push(npc_loc);

                        let mut mob = to_entity(
                            npc_def,
                            npc_loc.xy,
                        );

                        mob.inventory.push(
                            to_entity(
                                item_spec.item_def,
                                npc_loc.xy,
                            )
                        );

                        world.mobs.insert(
                            segment_id,
                            mob,
                        );

                        break
                    }
                }
            }
        }

        if attempts >= 16 {
            let needle: &models::config::WorldSegment = &config_segments[usize::from(last_attempted_segment_id)];
            let mut config_index = 0;
            for i in 0..config.segments.len() {
                if &config.segments[i] == needle {
                    config_index = i;
                    break
                }
            }

            return Err(Error::CouldNotPlaceItem {
                def: item_spec.item_def.to_owned(),
                config_index,
            });
        }
    }

    assert_door_targets_seem_right!();


    #[cfg(feature = "invariant-checking")]
    {
        // Assert that at least one victory door or door that transforms into a victory door
        // is in the world.

        let mut found = false;

        for entity in world.all_entities_mut() {
            // Just using the mut version because the other version does not exist as of this writing.
            let entity = &entity;

            if entity.is_door() {
                if entity.is_victory() {
                    found = true;
                    break
                } else if item_defs.iter().any(|item|
                        item.on_collect.iter()
                        .find(|action|
                            match action {
                                CollectAction::Transform(transform) => {
                                    find_def_if_locked_victory_door(&door_defs, *transform)
                                        .map(|def| def.id == entity.def_id())
                                        .unwrap_or(false)
                                }
                            }
                        ).is_some()
                    ) {
                    found = true;
                    break
                } else {
                    // Not an interesting entity.
                }
            } else {
                // Not an interesting entity.
            }
        }

        invariant_assert!(found, "No way to win was found!");
    }

    Ok(Generated{
        world,
        speeches,
        inventory_descriptions,
        entity_defs,
        goal_door_tile_sprite,
        hallway_states,
    })
}