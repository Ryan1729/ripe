
use rhai::{Engine, EvalAltResult};

use std::sync::LazyLock;

use game::{Config, Tile, WorldSegment};

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
    SizeError(std::num::TryFromIntError),
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

    const TILE_FLAGS: &str = r#"
        export const WALL = 0; // Can't be anything but a blocker
        export const FLOOR = 1 << 0;
        export const PLAYER_START = 1 << 2;
        export const ITEM_START = 1 << 3;
        export const NPC_START = 1 << 4;
    "#;

    let tile_flags_ast = engine.compile(TILE_FLAGS)
        .expect("TILE_FLAGS should compile");
    let tile_flags_module = Module::eval_ast_as_new(Scope::new(), &tile_flags_ast, &engine)
        .expect("TILE_FLAGS should eval as a module");

    let mut resolver = StaticModuleResolver::new();
    
    resolver.insert("tile_flags", tile_flags_module);

    engine.set_module_resolver(resolver);

    engine
});

pub fn parse(code: &str) -> Result<Config, Error> {
    let map: rhai::Map = ENGINE.eval(code)?;

    let segments = {
        let key = "segments";
        map.get(key)
            .ok_or(Error::FieldMissing{ key, parent_key: ik!("#root"), })?
            .as_array_ref().map_err(|got| Error::TypeMismatch{ key: ik!(key), expected: "array", got })?
    };

    let mut config = Config {
        segments: Vec::with_capacity(segments.len()),
    };

    for i in 0..segments.len() {
        let parent_key = ik!("segments", i);

        let segment = segments[i]
            .as_map_ref().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

        let width = {
            let key = "width";
            segment.get(key)
                .ok_or(Error::FieldMissing{ key, parent_key, })?
                .as_int().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "int", got })?
                .try_into().map_err(Error::SizeError)?
        };

        let tiles = {
            let key = "tiles";
            let raw_tiles = segment.get(key)
                .ok_or(Error::FieldMissing{ key, parent_key, })?
                .as_array_ref().map_err(|got| Error::TypeMismatch{ key: ik!(key), expected: "array", got })?;

            let mut tiles: Vec<game::Tile> = Vec::with_capacity(raw_tiles.len());

            for i in 0..raw_tiles.len() {
                let tile = match raw_tiles[i]
                    .as_int().map_err(|got| Error::TypeMismatch{ key: ik!(key, i), expected: "int", got })? {
                    0 => Tile { sprite: 0 },
                    1..31 => Tile { sprite: 1 },
                    got => { return Err(Error::UnexpectedTileKind { index: i, got, }); },
                };

                tiles.push(tile);
            }

            tiles
        };

        config.segments.push(WorldSegment {
            width,
            tiles,
        });
    }

    Ok(config)
}
