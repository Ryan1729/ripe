use models::{config::{Config}, speeches, DefId, Entity, CollectAction, Location, MiniEntityDef, Speeches, Tile, TileSprite, WorldSegment, X, Y, SegmentId};
use models::consts::{ITEM_START, NPC_START, PLAYER_START, COLLECTABLE, STEPPABLE, NOT_SPAWNED_AT_START, DOOR, FLOOR, DOOR_START};
use vec1::Vec1;
use xs::{Xs};

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

pub fn transform_all_matching(world: &mut World, from_def_id: DefId, to_def: &MiniEntityDef) {
    for entity in world.all_entities_mut() {
        if entity.def_id == from_def_id {
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
    let mut segments = Vec::with_capacity(16);
    let mut config_segments = Vec::with_capacity(16);

    // TODO randomize the amount here
    for _ in 0..4 {
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

    xs::shuffle(rng, &mut mob_defs);
    xs::shuffle(rng, &mut item_defs);
    xs::shuffle(rng, &mut door_defs);

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

    let p_loc = random::tile_matching_flags(rng, first_config_segment, first_segment_id, PLAYER_START)
        .or_else(||
            random::passable_tile(rng, first_segment)
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
                    rng,
                    config_segment_1,
                    i,
                    FLOOR | DOOR_START,
                    &placed_already,
                ).ok_or(Error::CannotPlaceDoor)?;
                placed_already.push(d_1_loc);

                let d_2_loc = random::tile_matching_flags_besides(
                    rng,
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
    let index_offset = xs::range(rng, 0..item_defs.len() as u32) as usize;
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
                        rng,
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
        world: &World,
        all_desires: &[DesireRef<'defs>],
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

            // TODO: have multiple spheres, delineated by locked doors.
            // Treat each sphere as a separate puzzle, with unlocking
            // the next door as the goal.
            // Keep track of the used desires, so we don't use the same one twice.
            // We can randomly push some things back into previous spheresm for variety.

            // TODO Determine the available segments from the current sphere
            debug_assert!(world.segments.len() <= SegmentId::MAX as usize);
            let segment_ids = (0..world.segments.len() as SegmentId).collect::<Vec<_>>();
            debug_assert!(segment_ids.len() <= u32::MAX as usize);

            macro_rules! random_segment_id {
                () => {
                    segment_ids[xs::range(rng, 0..segment_ids.len() as u32) as usize]
                }
            }

            // Start with a solvable puzzle, then add steps, keeping it solvable
            item_specs.push(ItemSpec{
                item_def: goal_item_def,
                location: AbstractLocation::Floor(random_segment_id!()),
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

        Constraints {
            item_specs,
        }
    }

    let constraints: Constraints = select_constraints(rng, &world, &all_desires, goal_item_def);

    for item_spec in constraints.item_specs {
        let mut attempts = 0;

        while attempts < 16 {
            attempts += 1;

            use AbstractLocation::*;
            match item_spec.location {
                Floor(segment_id) => {
                    if let Some(item_loc) = random::tile_matching_flags_besides(
                        rng,
                        &config_segments[usize::from(segment_id)],
                        segment_id,
                        ITEM_START,
                        &placed_already,
                    ) {
                        world.steppables.insert(
                            item_loc.id,
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
            return Err(Error::CouldNotPlaceItem(item_spec.item_def.to_owned()));
        }
    }

    Ok(Generated{
        world,
        speeches,
        inventory_descriptions,
        entity_defs,
        goal_door_tile_sprite,
    })
}