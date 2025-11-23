
use rhai::{Engine, EvalAltResult};

use std::sync::LazyLock;

//use models::{};
use game::{Config};
use game::config::{TileFlags, WorldSegment};

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
}

impl From<Box<EvalAltResult>> for Error {
    fn from(ear: Box<EvalAltResult>) -> Self {
        Self::EvalAltResult(ear)
    }
}

static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
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

    use game::to_tile::TILES_PER_ROW;

    // Rhai not allowing you to access consts outside the fucntion scope withotu using `function_name!` is annoying.
    let default_spritesheet_string = format!(r#"
        private fn tile_sprite_n_at_offset(n, offset) {{
            const TILES_PER_ROW = {TILES_PER_ROW};
            n * TILES_PER_ROW + offset
        }}

        fn mob(n) {{
            tile_sprite_n_at_offset(n, 3)
        }}

        fn item(n) {{
            tile_sprite_n_at_offset(n, 4)
        }}
    "#);

    add_module!(default_spritesheet = default_spritesheet_string);

    engine.set_module_resolver(resolver);

    engine
});

pub fn parse(code: &str) -> Result<Config, Error> {
    use models::{DefId, Speech};
    use game::{EntityDef};

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
        let parent_key = ik!("entities", id.into());

        let entity = entities[usize::from(id)]
            .as_map_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

        let flags = get_int!(entity, "flags", parent_key);

        let tile_sprite = get_int!(entity, "tile_sprite", parent_key);

        let speeches = 'speeches: {
            let key = "speeches";
            let raw_speeches = match entity.get(key) {
                None => break 'speeches vec![],
                Some(dynamic) => dynamic
                    .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?
            };

            let mut speeches = Vec::with_capacity(raw_speeches.len());

            for i in 0..raw_speeches.len() {
                let text = raw_speeches[i].clone()
                    .into_string().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "string", got })?;

                speeches.push(Speech {
                    text,
                });
            }

            speeches
        };

        let inventory_description = 'inventory_description: {
            let key = "inventory_description";
            let raw_inventory_description = match entity.get(key) {
                None => break 'inventory_description vec![],
                Some(dynamic) => dynamic
                    .as_array_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "array", got })?
            };

            let mut inventory_description = Vec::with_capacity(raw_inventory_description.len());

            for i in 0..raw_inventory_description.len() {
                let text = raw_inventory_description[i].clone()
                    .into_string().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "string", got })?;

                inventory_description.push(Speech {
                    text,
                });
            }

            inventory_description
        };

        entities_vec.push(EntityDef {
            flags,
            speeches,
            inventory_description,
            id,
            tile_sprite,
        });
    }

    Ok(Config {
        segments: segments_vec.try_into().map_err(|_| Error::NoSegmentsFound)?,
        entities: entities_vec.try_into().map_err(|_| Error::NoEntitiesFound)?,
    })
}
