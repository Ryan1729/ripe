
use rhai::{Engine, EvalAltResult};

use std::sync::LazyLock;

use models::{DefId, DefIdDelta};
use game::{Config};
use game::config::{TileFlags, WorldSegment};

pub const TILES_PER_ROW: u8 = 8;

#[derive(Clone, Copy, Debug)]
pub struct IndexableKey {
    pub key: &'static str,
    pub index: Option<usize>,
}

macro_rules! ik {
    ($key: expr) => {
        IndexableKey {
            key: $key,
            index: None,
        }
    };
    ($key: expr ,$index: expr) => {
        IndexableKey {
            key: $key,
            index: Some($index),
        }
    };
}

// TODO: Instead of ad hoc fields deciding what context seems relevant to keep, we should just track the whole
// key/index chain down from the root of the map we get from the rhai evaluation. Each error would then have a
// ctx field of a new ErrorContext type.
// I think usage code should look like this
/*
let field = {
    let key = "a_key";
    let ctx = ctx.add_key(key);

    ... Err(Error::SomeVariant{ ctx, ... })?

    for i in 0..sub_things.len() {
        let ctx = ctx.add_index(i);

        ... Err(Error::SomeVariant{ ctx, ... })?
    }

}
*/
// One can imagine inplmenting a version of this API that performs no unneeded allocations, and maybe even
// reclaims things with destructors, if we were to ever really care about that perf.

#[derive(Debug)]
pub enum Error {
    EvalAltResult(Box<EvalAltResult>),
    TypeMismatch {
        key: IndexableKey,
        expected: &'static str,
        got: &'static str,
    },
    FieldMissing {
        key: &'static str,
        parent_key: IndexableKey,
    },
    UnexpectedTileKind {
        index: usize,
        got: rhai::INT,
    },
    UnexpectedEntityKind {
        index: usize,
        got: rhai::INT,
    },
    SizeError {
        key: &'static str,
        parent_key: IndexableKey,
        error: std::num::TryFromIntError,
    },
    TooManyEntityDefinitions{ got: usize },
    NoSegmentsFound,
    NoEntitiesFound,
    OutOfBoundsDefId {
        key: &'static str,
        parent_key: IndexableKey,
        def_id: models::DefId,
    },
    UnknownEntityDefIdRefKind {
        key: &'static str,
        parent_key: IndexableKey,
        kind: game::config::EntityDefIdRefKind,
    },
    UnknownCollectActionKind {
        key: &'static str,
        parent_key: IndexableKey,
        kind: game::config::CollectActionKind,
    },
    DefIdOverflow{
        key: &'static str,
        parent_key: IndexableKey,
        base: DefId,
        delta: DefIdDelta,
    }
}

impl From<Box<EvalAltResult>> for Error {
    fn from(ear: Box<EvalAltResult>) -> Self {
        Self::EvalAltResult(ear)
    }
}

fn init_engine() -> Engine {
    use rhai::{Module, Scope};
    use rhai::module_resolvers::StaticModuleResolver;

    let mut engine = Engine::new();

    let mut resolver = StaticModuleResolver::new();

    macro_rules! add_module {
        ($name: ident = $string: expr) => {{
            let $name: &str = &$string;

            let ast = engine.compile($name)
                .expect(concat!(stringify!($name), " should compile"));
            let module = Module::eval_ast_as_new(Scope::new(), &ast, &engine)
                .expect(concat!(stringify!($name), " should eval as a module"));

            resolver.insert(stringify!($name), module);
        }};
    }

    let mut tile_flags_string = String::with_capacity(128);

    for (name, value) in game::config::ALL_TILE_FLAGS {
        tile_flags_string += &format!("export const {name} = {value};\n");
    }

    add_module!(tile_flags = tile_flags_string);

    let mut entity_flags_string = String::with_capacity(128);

    for (name, value) in game::config::ALL_ENTITY_FLAGS {
        entity_flags_string += &format!("export const {name} = {value};\n");
    }

    add_module!(entity_flags = entity_flags_string);

    // Rhai not allowing you to access consts outside the function scope without using `function_name!` is annoying.
    let default_spritesheet_string = format!(r#"
        private fn tile_sprite_xy(x, y) {{
            const TILES_PER_ROW = {TILES_PER_ROW};
            y * TILES_PER_ROW + x
        }}

        fn mob(n) {{
            tile_sprite_xy(3, n)
        }}

        fn item(n) {{
            tile_sprite_xy(4, n)
        }}
        // TODO: Define walls, floor, door animation, and player here too

        export const OPEN_DOOR = tile_sprite_xy(0, 1);
        export const LOCKED_DOOR_1 = tile_sprite_xy(0, 2);
        export const LOCKED_DOOR_2 = tile_sprite_xy(0, 3);

        export const KEY_1 = tile_sprite_xy(1, 2);
        export const KEY_2 = tile_sprite_xy(1, 3);
    "#);

    add_module!(default_spritesheet = default_spritesheet_string);

    let mut entity_ids_string = String::with_capacity(256);

    for (name, value) in game::config::ALL_ENTITY_ID_REFERENCE_KINDS {
        entity_ids_string += &format!("export const {name} = {value};\n");
    }

    entity_ids_string += r#"
        fn relative(n) {
    "#;

    for (name, value) in game::config::ALL_ENTITY_ID_REFERENCE_KINDS {
        entity_ids_string += &format!("const {name} = {value};\n");
    }

    entity_ids_string += r#"
            #{
                kind: RELATIVE,
                value: n,
            }
        }
    "#;

    entity_ids_string += r#"
        fn absolute(n) {
    "#;

    for (name, value) in game::config::ALL_ENTITY_ID_REFERENCE_KINDS {
        entity_ids_string += &format!("const {name} = {value};\n");
    }

    entity_ids_string += r#"
            #{
                kind: ABSOLUTE,
                value: n,
            }
        }
    "#;

    add_module!(entity_ids = entity_ids_string);

    let mut collect_actions_string = String::with_capacity(256);

    for (name, value) in game::config::ALL_COLLECT_ACTION_KINDS {
        collect_actions_string += &format!("export const {name} = {value};\n");
    }

    add_module!(collect_actions = collect_actions_string);

    engine.set_module_resolver(resolver);

    engine
}

#[test]
fn init_engine_does_not_panic() {
    init_engine();
}

static ENGINE: LazyLock<Engine> = LazyLock::new(init_engine);

pub fn parse(code: &str) -> Result<Config, Error> {
    use std::ops::Deref;
    use models::{DefId, Speech, CollectAction};
    use game::{EntityDef, config::{EntityDefIdRefKind, CollectActionKind}};

    macro_rules! convert_to {
        ($from: expr => $to: ty, $key: expr, $parent_key: expr) => {
            <$to>::try_from($from).map_err(|error| Error::SizeError {
                key: $key,
                parent_key: $parent_key,
                error,
            })?
        }
    }

    macro_rules! get_int {
        ($map: expr, $key: expr, $parent_key: expr) => {
            {
                let key = $key;
                let parent_key = $parent_key;
                $map.get(key)
                    .ok_or(Error::FieldMissing{ key, parent_key, })?
                    .as_int().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "int", got })?
                    .try_into().map_err(|error| Error::SizeError {
                        key,
                        parent_key,
                        error,
                    })?
            }
        }
    }

    macro_rules! get_map {
        ($map: expr, $key: expr, $parent_key: expr) => {
            {
                let key = $key;
                let parent_key = $parent_key;
                $map.get(key)
                    .ok_or(Error::FieldMissing{ key, parent_key, })?
                    .as_map_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?
            }
        }
    }

    let map: rhai::Map = ENGINE.eval(code)?;

    let segments = {
        let key = "segments";
        map.get(key)
            .ok_or(Error::FieldMissing{ key, parent_key: ik!("#root"), })?
            .as_array_ref().map_err(|got| Error::TypeMismatch{ key: ik!(key), expected: "array", got })?
    };

    let entities = {
        let key = "entities";
        map.get(key)
            .ok_or(Error::FieldMissing{ key, parent_key: ik!("#root"), })?
            .as_array_ref().map_err(|got| Error::TypeMismatch{ key: ik!(key), expected: "array", got })?
    };

    let mut segments_vec = Vec::with_capacity(segments.len());
    let mut entities_vec = Vec::with_capacity(entities.len());

    for i in 0..segments.len() {
        let parent_key = ik!("segments", i);

        let segment = segments[i]
            .as_map_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

        let width = get_int!(segment, "width", parent_key);

        let tiles = {
            let key = "tiles";
            let raw_tiles = segment.get(key)
                .ok_or(Error::FieldMissing{ key, parent_key, })?
                .as_array_ref().map_err(|got| Error::TypeMismatch{ key: ik!(key), expected: "array", got })?;

            let mut tiles: Vec<TileFlags> = Vec::with_capacity(raw_tiles.len());

            for i in 0..raw_tiles.len() {
                let got = raw_tiles[i]
                    .as_int().map_err(|got| Error::TypeMismatch{ key: ik!(key, i), expected: "int", got })?;

                let tile_flags = match TileFlags::try_from(got) {
                    Ok(tf) => tf,
                    Err(_) => { return Err(Error::UnexpectedTileKind { index: i, got }); },
                };

                tiles.push(tile_flags);
            }

            tiles
        };

        segments_vec.push(WorldSegment {
            width,
            tiles,
        });
    }

    let entity_def_count = DefId::try_from(entities.len())
        .map_err(|_| Error::TooManyEntityDefinitions{ got: entities.len() })?;

    for id in 0..entity_def_count {
        fn deref_def_id(
            base: DefId,
            map: impl Deref<Target = rhai::Map>,
            entity_def_count: DefId,
            parent_key: IndexableKey,
        ) -> Result<DefId, Error> {
            let kind: EntityDefIdRefKind = get_int!(*map, "kind", parent_key);

            let key = "value";

            let value: models::DefIdNextLargerSigned = get_int!(*map, key, parent_key);

            let def_id = match kind {
                game::config::RELATIVE => {
                    let delta = convert_to!(value => DefIdDelta, key, parent_key);

                    base.checked_add_signed(delta).ok_or(Error::DefIdOverflow{ key, parent_key, base, delta })?
                },
                game::config::ABSOLUTE => convert_to!(value => DefId, key, parent_key),
                _ => return Err(Error::UnknownEntityDefIdRefKind { key, parent_key, kind }),
            };

            // TODO? Validate that the target is a valid kind of entity here?
            if def_id >= entity_def_count {
                return Err(Error::OutOfBoundsDefId{ key, parent_key, def_id });
            }

            Ok(def_id)
        }

        let parent_key = ik!("entities", id.into());

        let entity = entities[usize::from(id)]
            .as_map_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

        let flags = get_int!(entity, "flags", parent_key);

        let tile_sprite = get_int!(entity, "tile_sprite", parent_key);

        let speeches: Vec<Vec<Speech>> = 'speeches: {
            let key = "speeches";
            let raw_speeches_list = match entity.get(key) {
                None => break 'speeches vec![],
                Some(dynamic) => dynamic
                    .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?
            };

            let mut speeches = Vec::with_capacity(raw_speeches_list.len());

            for list_i in 0..raw_speeches_list.len() {
                let raw_speeches = raw_speeches_list[list_i]
                .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?;

                let mut individual_speeches = Vec::with_capacity(raw_speeches.len());

                for i in 0..raw_speeches.len() {
                    let raw_text = raw_speeches[i].clone()
                        .into_string().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "string", got })?;

                    // TODO? Allow avoiding this reflow per speech?
                    individual_speeches.push(Speech::from(&raw_text));
                }

                speeches.push(individual_speeches);
            }

            speeches
        };

        let inventory_description: Vec<Vec<Speech>> = 'inventory_description: {
            let key = "inventory_description";
            let raw_inventory_description_list = match entity.get(key) {
                None => break 'inventory_description vec![],
                Some(dynamic) => dynamic
                    .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?
            };

            let mut inventory_description = Vec::with_capacity(raw_inventory_description_list.len());

            for list_i in 0..raw_inventory_description_list.len() {
                let raw_inventory_description = raw_inventory_description_list[list_i]
                .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?;

                let mut individual_inventory_description = Vec::with_capacity(raw_inventory_description.len());

                for i in 0..raw_inventory_description.len() {
                    let raw_text = raw_inventory_description[i].clone()
                        .into_string().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "string", got })?;

                    // TODO? Allow avoiding this reflow per speech?
                    individual_inventory_description.push(Speech::from(&raw_text));
                }

                inventory_description.push(individual_inventory_description);
            }

            inventory_description
        };

        let wants = 'wants: {
            let key = "wants";
            let raw_wants = match entity.get(key) {
                None => break 'wants vec![],
                Some(dynamic) => dynamic
                    .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?
            };

            let want_count: DefId = convert_to!(raw_wants.len() => DefId, key, parent_key);

            let mut wants = Vec::with_capacity(raw_wants.len());

            for i in 0..want_count {
                let parent_key = ik!("wants", i.into());

                let map = raw_wants[i as usize]
                        .as_map_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

                let def_id = deref_def_id(
                    id,
                    map,
                    entity_def_count,
                    parent_key,
                )?;

                wants.push(def_id);
            }

            wants
        };

        let on_collect = 'on_collect: {
            let key = "on_collect";
            let raw_on_collect = match entity.get(key) {
                None => break 'on_collect vec![],
                Some(dynamic) => dynamic
                    .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?
            };

            let on_collect_count: DefId = convert_to!(raw_on_collect.len() => DefId, key, parent_key);

            let mut on_collect = Vec::with_capacity(raw_on_collect.len());

            for i in 0..on_collect_count {
                let action_map = raw_on_collect[i as usize]
                    .as_map_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

                let parent_key = ik!("on_collect", i.into());

                let kind: CollectActionKind = get_int!(action_map, "kind", parent_key);

                match kind {
                    game::config::TRANSFORM => {
                        let from_map = get_map!(action_map, "from", parent_key);

                        let from = deref_def_id(
                            id,
                            from_map,
                            entity_def_count,
                            parent_key,
                        )?;

                        let to_map = get_map!(action_map, "to", parent_key);

                        let to = deref_def_id(
                            id,
                            to_map,
                            entity_def_count,
                            parent_key,
                        )?;

                        on_collect.push(CollectAction::Transform { from, to });
                    },
                    _ => return Err(Error::UnknownCollectActionKind { key, parent_key, kind }),
                }


            }

            on_collect
        };

        entities_vec.push(EntityDef {
            flags,
            speeches,
            inventory_description,
            id,
            tile_sprite,
            wants,
            on_collect,
        });
    }

    Ok(Config {
        segments: segments_vec.try_into().map_err(|_| Error::NoSegmentsFound)?,
        entities: entities_vec.try_into().map_err(|_| Error::NoEntitiesFound)?,
    })
}
