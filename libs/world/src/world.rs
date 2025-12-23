use features::{invariant_assert};
use models::{config::{Config}, speeches, CollectAction, DefId, Entity, EntityTransformable, Location, MiniEntityDef, Speeches, Tile, TileSprite, Transform, WorldSegment, X, Y, SegmentId};
use models::consts::{ITEM_START, NPC_START, PLAYER_START, COLLECTABLE, STEPPABLE, VICTORY, NOT_SPAWNED_AT_START, DOOR, FLOOR, DOOR_START};
use vec1::Vec1;
use xs::{Xs};

mod entities {
    use models::{Entity, X, Y, XY, SegmentId};

    use std::collections::{BTreeMap};

    pub type Key = models::Location;

    pub fn entity_key(segment_id: SegmentId, x: X, y: Y) -> Key {
        Key {
            segment_id,
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

pub type EntityKey = entities::Key;

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

    pub fn player_key(&self) -> EntityKey {
        entity_key(
            self.segment_id,
            self.player.x,
            self.player.y,
        )
    }

    pub fn local_key(&self, x: X, y: Y) -> EntityKey {
        entity_key(
            self.segment_id,
            x,
            y,
        )
    }

    pub fn get_entity(&self, key: EntityKey) -> Option<&Entity> {
        if key == self.player_key() {
            return Some(&self.player)
        }

        self.mobs.get(key).or_else(|| self.steppables.get(key))
    }

    pub fn get_entity_mut(&mut self, key: EntityKey) -> Option<&mut Entity> {
        if key == self.player_key() {
            return Some(&mut self.player)
        }

        self.mobs.get_mut(key).or_else(|| self.steppables.get_mut(key))
    }
}

pub fn to_entity(
    def: &MiniEntityDef,
    x: X,
    y: Y
) -> Entity {
    Entity::new(
        x,
        y,
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
        is_passable,
        WorldSegment,
    };
    use xs::{Xs};

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
                    door_defs: &[SOME_LOCKED_DOOR],
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
}

pub fn generate(rng: &mut Xs, config: &Config) -> Result<Generated, Error> {
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
        let index = xs::range(rng, 0..config.segments.len() as u32) as usize;

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

    let mut mob_defs: Vec1<_> = mob_defs.try_into().map_err(|_| Error::NoMobsFound)?;
    let mut item_defs: Vec1<_> = item_defs.try_into().map_err(|_| Error::NoItemsFound)?;
    let mut door_defs: Vec1<_> = door_defs.try_into().map_err(|_| Error::NoDoorsFound)?;

    xs::shuffle(rng, &mut mob_defs);
    drop(mob_defs); // Wait, do we need this?
    xs::shuffle(rng, &mut item_defs);
    xs::shuffle(rng, &mut door_defs);

    let player = Entity {
        transformable: EntityTransformable {
            tile_sprite: models::PLAYER_SPRITE,
            ..<_>::default()
        },
        ..<_>::default()
    };

    let mut world = World {
        player,
        segment_id: <_>::default(),
        segments,
        steppables: <_>::default(),
        mobs: <_>::default(),
    };

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
    world.player.x = p_loc.xy.x;
    world.player.y = p_loc.xy.y;
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
    let index_offset = xs::range(rng, 0..item_defs.len() as u32) as usize;
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
                        d_loc.xy.x,
                        d_loc.xy.y
                    );
                    // We don't need to set targets for victory doors.

                    world.steppables.insert(
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
        let initial_lak_index = xs::range(rng, 0..non_final_lock_and_keys.len() as u32) as usize;
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

        for chunk_index in 0..chunks_len {
            let chunk = &chunks[chunk_index];

            // Connect up all the other segments in the chunk with open doors
            for window in chunk
                .windows(2)
                .chain(std::iter::once([chunk[chunk.len() - 1], chunk[0]].as_slice())) {
                assert_eq!(window.len(), 2);

                let segment_id_i = window[0];
                let segment_id_j = window[1];

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
                    open_door_def,
                    d_i_loc.xy.x,
                    d_i_loc.xy.y
                );
                door_i.door_target = d_j_loc;
                track_door_target_set!(door_i);

                world.steppables.insert(segment_id_i, door_i);

                assert_door_targets_seem_right!();

                let mut door_j = to_entity(
                    open_door_def,
                    d_j_loc.xy.x,
                    d_j_loc.xy.y
                );
                door_j.door_target = d_i_loc;
                track_door_target_set!(door_j);

                world.steppables.insert(segment_id_j, door_j);

                assert_door_targets_seem_right!();
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

            {
                let segment_id = {
                    let chunk = &chunks[chunk_index];
                    chunk[xs::range(rng, 0..chunk.len() as u32) as usize]
                };
                let next_chunk_segment_id = {
                    let chunk = &chunks[next_chunk_index];
                    chunk[xs::range(rng, 0..chunk.len() as u32) as usize]
                };

                let config_segment_1 = &config_segments[segment_id as usize];
                let config_segment_2 = &config_segments[next_chunk_segment_id as usize];

                let d_1_loc = random::tile_matching_flags_besides(
                    rng,
                    config_segment_1,
                    segment_id,
                    FLOOR | DOOR_START,
                    &placed_already,
                ).ok_or(Error::CannotPlaceDoor)?;
                placed_already.push(d_1_loc);

                let d_2_loc = random::tile_matching_flags_besides(
                    rng,
                    config_segment_2,
                    next_chunk_segment_id,
                    FLOOR | DOOR_START,
                    &placed_already,
                ).ok_or(Error::CannotPlaceDoor)?;
                placed_already.push(d_2_loc);

                let mut door_1 = to_entity(
                    edge_lak.lock,
                    d_1_loc.xy.x,
                    d_1_loc.xy.y
                );
                door_1.door_target = d_2_loc;
                track_door_target_set!(door_1);

                world.steppables.insert(segment_id, door_1);

                let mut door_2 = to_entity(
                    edge_lak.lock,
                    d_2_loc.xy.x,
                    d_2_loc.xy.y
                );
                door_2.door_target = d_1_loc;
                track_door_target_set!(door_2);

                world.steppables.insert(next_chunk_segment_id, door_2);
            }

            assert_door_targets_seem_right!();

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
            for (_, steppable) in world.steppables.for_id(segment_id) {
                if steppable.is_door() {
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

        let desire_count_to_use = xs::range(rng, 1..all_desires.len() as u32 + 1) as usize;
        // We end up with 1 more spec than the desires we use, becauase we start with one that doesn't use any.
        let overall_target_len = desire_count_to_use + 1;

        let average_target_len = core::cmp::max(overall_target_len / spheres.len(), 1);
        let max_sub_target_len = average_target_len * 2;
        invariant_assert!(max_sub_target_len <= u32::MAX as usize); // For random selection later

        let initial_index = xs::range(rng, 0..all_desires.len() as u32) as usize;

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
                        segment_ids[xs::range(rng, 0..segment_ids.len() as u32) as usize]
                    }
                }

                // Start with a solvable puzzle, then add steps, keeping it solvable
                item_specs.push(ItemSpec{
                    item_def: sphere.goal_item_def,
                    location: AbstractLocation::Floor(random_segment_id!()),
                });

                let sub_target_len = xs::range(rng, 1..max_sub_target_len as u32) as usize;

                let mut index = xs::range(rng, 0..all_desires.len() as u32) as usize;

                let initial_spec_len = item_specs.len();

                while item_specs.len() - initial_spec_len < sub_target_len {
                    // Select the index or not, at a rate proportional to how many we need.
                    if (xs::range(rng, 0..all_desires.len() as u32 + 1) as usize) < sub_target_len {
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
                        world.steppables.insert(
                            item_loc.segment_id,
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
    })
}