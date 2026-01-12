//use hardcoded as used_mod;
use rune_based as used_mod;

pub use used_mod::{parse, Error};

mod rune_based {
    use platform_types::TILES_PER_ROW;
    use models::{
        config::{
            Config,
            WorldSegment,
        },
        consts::{TileFlags},
        DefId,
        DefIdDelta
    };
    use rune::{alloc::{Error as AllocError}, BuildError, Context, ContextError, Diagnostics, Source, Sources, Vm};
    use rune::diagnostics::{Diagnostic};
    use rune::runtime::{Object, RuntimeError, VmError};
    use rune::sync::Arc;

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

    impl From<&'static str> for IndexableKey {
        fn from(key: &'static str) -> Self {
            ik!(key)
        }
    }

    impl core::fmt::Display for IndexableKey {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.key)?;

            if let Some(index) = self.index {
                write!(f, "[{index}]")
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug)]
    pub enum Error {
        Hardcoded(crate::hardcoded::Error),
        Alloc(AllocError),
        Build(BuildError),
        Context(ContextError),
        Diagnostics(Vec<Diagnostic>, Sources),
        Runtime(RuntimeError),
        Vm(VmError),
        /// An error the configuration evaluation itself returned.
        FromConfig(String),
        TypeMismatch {
            key: IndexableKey,
            expected: &'static str,
            got: RuntimeError,
        },
        FieldMissing {
            key: &'static str,
            parent_key: IndexableKey,
        },
        UnexpectedTileKind {
            index: usize,
            got: u64,
        },
        UnexpectedEntityKind {
            index: usize,
            got: u64,
        },
        SizeError {
            key: IndexableKey,
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
            kind: models::consts::EntityDefIdRefKind,
        },
        UnknownCollectActionKind {
            key: &'static str,
            parent_key: IndexableKey,
            kind: models::consts::CollectActionKind,
        },
        DefIdOverflow{
            key: &'static str,
            parent_key: IndexableKey,
            base: DefId,
            delta: DefIdDelta,
        },
        UnknownHallwayKind{
            key: &'static str,
            parent_key: IndexableKey,
            kind: models::consts::HallwayKind,
        },
    }

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use Error::*;
            match self {
                Diagnostics(diagnostics, sources) => {
                    use std::io::Write;
                    use rune::termcolor::{ColorSpec, WriteColor};
                    struct Wrapper<'mut_ref, 'formatter>(&'mut_ref mut core::fmt::Formatter<'formatter>);

                    impl Write for Wrapper<'_, '_> {
                        fn write(&mut self, bytes: &[u8]) -> Result<usize, std::io::Error> {
                            let len = bytes.len();
                            let s: &str = std::str::from_utf8(bytes).map_err(std::io::Error::other)?;
                            self.0.write_str(s).map(|()| len).map_err(std::io::Error::other)
                        }
                        fn flush(&mut self) -> Result<(), std::io::Error> {
                            Ok(())
                        }
                    }

                    impl WriteColor for Wrapper<'_, '_> {
                        fn supports_color(&self) -> bool { false }
                        fn set_color(&mut self, _spec: &ColorSpec) -> Result<(), std::io::Error> { Ok(()) }
                        fn reset(&mut self) -> Result<(), std::io::Error> { Ok(()) }
                    }

                    let mut write_wrapper = &mut Wrapper(f);

                    for diagnostic in diagnostics {
                        match {
                            match diagnostic {
                                Diagnostic::Fatal(d) => d.emit(write_wrapper, sources),
                                Diagnostic::Warning(d) => d.emit(write_wrapper, sources),
                                Diagnostic::RuntimeWarning(d) => d.emit(write_wrapper, sources, None, None),
                                other => {
                                    write!(write_wrapper.0, "Unknown `Diagnostic` variant: {other:#?}")?;
                                    Ok(())
                                }
                            }
                        } {
                            Ok(()) => {},
                            Err(e) => write!(write_wrapper.0, "emit error: {e:#?}")?,
                        }
                    }

                    Ok(())
                },
                Runtime(e) => {
                    write!(f, " Rune RuntimeError:\n  {e}")
                }
                Vm(e) => {
                    write!(f, " Rune VmError:\n  {e}")
                }
                FromConfig(e) => {
                    write!(f, " FromConfig Error:\n  {e}")
                }
                TypeMismatch {
                    key,
                    expected,
                    got,
                } => {
                    write!(f, " TypeMismatch @ {key}, expected {expected}:\n  {got}")
                }
                // TODO implement proper human readable display here
                _ => write!(f, " fmt::Debug Fallback:\n  {self:#?}"),
            }
        }
    }

    impl From<AllocError> for Error {
        fn from(e: AllocError) -> Self {
            Self::Alloc(e)
        }
    }

    impl From<BuildError> for Error {
        fn from(e: BuildError) -> Self {
            Self::Build(e)
        }
    }

    impl From<ContextError> for Error {
        fn from(e: ContextError) -> Self {
            Self::Context(e)
        }
    }

    impl From<VmError> for Error {
        fn from(e: VmError) -> Self {
            Self::Vm(e)
        }
    }

    pub fn parse(code: &str) -> Result<Config, Error> {
        let map: Object = eval(code)?;

        to_config(map)
    }

    fn to_config(map: Object) -> Result<Config, Error> {
        use std::ops::Deref;
        use rune::runtime::{BorrowRef, Object};
        use rune::{Value};
        use models::{
            consts::{EntityDefIdRefKind, CollectActionKind},
            CollectAction,
            DefId,
            EntityDef,
            SegmentWidth,
            Speech
        };

        macro_rules! convert_to {
            ($from: expr => $to: ty, $key: expr, $parent_key: expr $(,)?) => {
                <$to>::try_from($from).map_err(|error| Error::SizeError {
                    key: $key.into(),
                    parent_key: $parent_key,
                    error,
                })?
            }
        }

        macro_rules! to_int {
            ($val: expr, $key: expr, $parent_key: expr $(,)?) => ({
                let key = $key;
                let parent_key = $parent_key;

                let int: i64 = 
                    $val.as_integer().map_err(|got| Error::TypeMismatch{ key, expected: "int", got })?;

                int
                    .try_into().map_err(|error| Error::SizeError {
                        key,
                        parent_key,
                        error,
                    })?
            })
        }

        macro_rules! to_array {
            ($val: expr, $key: expr $(,)?) => ({
                let vec: Vec<Value> = rune::from_value(
                    $val
                ).map_err(|got| Error::TypeMismatch{ key: $key, expected: "array", got })?;

                vec
            })
        }

        macro_rules! get_int {
            ($map: expr, $key: expr, $parent_key: expr $(,)?) => {
                {
                    let key = $key;
                    let parent_key = $parent_key;

                    let value: &Value =
                        $map.get(key)
                            .ok_or(Error::FieldMissing{ key, parent_key, })?;

                    to_int!(
                        value,
                        ik!(key),
                        parent_key,
                    )
                }
            }
        }

        macro_rules! get_map {
            ($map: expr, $key: expr, $parent_key: expr $(,)?) => {
                {
                    let key = $key;
                    let parent_key = $parent_key;
                    let obj: Object =
                        rune::from_value(
                            $map.get(key)
                                .ok_or(Error::FieldMissing{ key, parent_key, })?
                        ).map_err(|got| Error::TypeMismatch{ key: key.into(), expected: "map", got })?;

                    obj
                }
            }
        }

        macro_rules! get_array {
            ($map: expr, $key: expr, $parent_key: expr $(,)?) => {
                {
                    let key = $key;
                    let parent_key = $parent_key;

                    to_array!(
                        $map.get(key)
                            .ok_or(Error::FieldMissing{ key, parent_key, })?,
                        ik!(key)
                    )
                }
            }
        }

        let segments = get_array!(map, "segments", ik!("#root"));

        let mut segments_vec = Vec::with_capacity(segments.len());

        for i in 0..segments.len() {
            let parent_key = ik!("segments", i);

            let segment: Object = rune::from_value(segments[i].clone())
                .map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

            let width: SegmentWidth = get_int!(segment, "width", parent_key);

            let tiles = {
                let key = "tiles";
                let raw_tiles = get_array!(segment, key, parent_key);

                let mut tiles: Vec<TileFlags> = Vec::with_capacity(raw_tiles.len());

                for i in 0..raw_tiles.len() {
                    let got = to_int!(raw_tiles[i], ik!(key, i), parent_key);

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

        let segments = segments_vec.try_into().map_err(|_| Error::NoSegmentsFound)?;

        let entities = get_array!(map, "entities", ik!("#root"));

        let mut entities_vec = Vec::with_capacity(entities.len());

        let entity_def_count = DefId::try_from(entities.len())
            .map_err(|_| Error::TooManyEntityDefinitions{ got: entities.len() })?;

        for id in 0..entity_def_count {
            fn deref_def_id(
                base: DefId,
                map: &Object,
                entity_def_count: DefId,
                parent_key: IndexableKey,
            ) -> Result<DefId, Error> {
                let kind: EntityDefIdRefKind = get_int!(*map, "kind", parent_key);

                let key = "value";

                let value: models::DefIdNextLargerSigned = get_int!(*map, key, parent_key);

                let def_id = match kind {
                    models::consts::RELATIVE => {
                        let delta = convert_to!(value => DefIdDelta, key, parent_key);

                        base.checked_add_signed(delta).ok_or(Error::DefIdOverflow{ key, parent_key, base, delta })?
                    },
                    models::consts::ABSOLUTE => convert_to!(value => DefId, key, parent_key),
                    _ => return Err(Error::UnknownEntityDefIdRefKind { key, parent_key, kind }),
                };

                // TODO? Validate that the target is a valid kind of entity here?
                if def_id >= entity_def_count {
                    return Err(Error::OutOfBoundsDefId{ key, parent_key, def_id });
                }

                Ok(def_id)
            }

            let parent_key = ik!("entities", id.into());

            let entity: Object = rune::from_value(entities[usize::from(id)].clone())
                .map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

            let flags = get_int!(entity, "flags", parent_key);

            let tile_sprite = get_int!(entity, "tile_sprite", parent_key);

            let speeches: Vec<Vec<Speech>> = 'speeches: {
                let key = "speeches";

                let raw_speeches_list = match entity.get(key) {
                    None => break 'speeches vec![],
                    Some(dynamic) => to_array!(dynamic, ik!(key)),
                };

                let mut speeches = Vec::with_capacity(raw_speeches_list.len());

                for list_i in 0..raw_speeches_list.len() {
                    let parent_key = ik!(key, list_i);

                    let raw_speeches = to_array!(raw_speeches_list[list_i].clone(), parent_key);

                    let mut individual_speeches = Vec::with_capacity(raw_speeches.len());

                    for i in 0..raw_speeches.len() {
                        let raw_text = raw_speeches[i].clone()
                            .into_string().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "string", got })?;

                        // TODO? Allow avoiding this reflow per speech?
                        individual_speeches.push(Speech::from(raw_text.as_str()));
                    }

                    speeches.push(individual_speeches);
                }

                speeches
            };

            let inventory_description: Vec<Vec<Speech>> = 'inventory_description: {
                let key = "inventory_description";

                let raw_inventory_description_list = match entity.get(key) {
                    None => break 'inventory_description vec![],
                    Some(dynamic) => to_array!(dynamic, ik!(key)),
                };

                let mut inventory_description = Vec::with_capacity(raw_inventory_description_list.len());

                for list_i in 0..raw_inventory_description_list.len() {
                    let parent_key = ik!(key, list_i);

                    let raw_inventory_description = to_array!(&raw_inventory_description_list[list_i], parent_key);

                    let mut individual_inventory_description = Vec::with_capacity(raw_inventory_description.len());

                    for i in 0..raw_inventory_description.len() {
                        let raw_text = raw_inventory_description[i].clone()
                            .into_string().map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "string", got })?;

                        // TODO? Allow avoiding this reflow per speech?
                        individual_inventory_description.push(Speech::from(raw_text.as_str()));
                    }

                    inventory_description.push(individual_inventory_description);
                }

                inventory_description
            };

            let wants = 'wants: {
                let key = "wants";
                
                let raw_wants = match entity.get(key) {
                    None => break 'wants vec![],
                    Some(dynamic) => to_array!(dynamic, ik!(key)),
                };

                let want_count: DefId = convert_to!(raw_wants.len() => DefId, key, parent_key);

                let mut wants = Vec::with_capacity(raw_wants.len());

                for i in 0..want_count {
                    let parent_key = ik!("wants", i.into());

                    let map: Object = rune::from_value(raw_wants[i as usize].clone())
                        .map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

                    let def_id = deref_def_id(
                        id,
                        &map,
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
                    Some(dynamic) => to_array!(dynamic, ik!(key)),
                };

                let on_collect_count: DefId = convert_to!(raw_on_collect.len() => DefId, key, parent_key);

                let mut on_collect = Vec::with_capacity(raw_on_collect.len());

                for i in 0..on_collect_count {
                    let action_map: Object = rune::from_value(raw_on_collect[i as usize].clone())
                        .map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

                    let parent_key = ik!("on_collect", i.into());

                    let kind: CollectActionKind = get_int!(action_map, "kind", parent_key);

                    match kind {
                        models::consts::TRANSFORM => {
                            let from = {
                                let key = "from";
                                    
                                let value: &Value = action_map.get(key)
                                    .ok_or(Error::FieldMissing{ key, parent_key, })?;
                                // We previously observed a "Cannot take" error when using `get_map!`,
                                // which is why we borrow instead.
                                let from_ref: BorrowRef<Object> = value.borrow_ref().map_err(Error::Runtime)?;
    
                                let from_map: &Object = from_ref.as_ref();
    
                                deref_def_id(
                                    id,
                                    from_map,
                                    entity_def_count,
                                    parent_key,
                                )?
                            };

                            let to = {
                                let key = "to";
                                    
                                let value: &Value = action_map.get(key)
                                    .ok_or(Error::FieldMissing{ key, parent_key, })?;
                                // We previously observed a "Cannot take" error when using `get_map!`,
                                // which is why we borrow instead.
                                let to_ref: BorrowRef<Object> = value.borrow_ref().map_err(Error::Runtime)?;
    
                                let to_map: &Object = to_ref.as_ref();
    
                                deref_def_id(
                                    id,
                                    to_map,
                                    entity_def_count,
                                    parent_key,
                                )?
                            };

                            on_collect.push(CollectAction::Transform(models::Transform{ from, to }));
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

        let entities = entities_vec.try_into().map_err(|_| Error::NoEntitiesFound)?;

        let hallways = get_array!(map, "hallways", ik!("#root"));

        let mut hallways_vec = Vec::with_capacity(hallways.len());

        for i in 0..hallways.len() {
            use models::config::HallwaySpec;
            let parent_key = ik!("hallways", i.into());

            let hallway: Object = rune::from_value(hallways[i].clone())
                .map_err(|got| Error::TypeMismatch{ key: parent_key, expected: "map", got })?;

            let key = "kind";

            let kind: models::consts::HallwayKind = get_int!(hallway, key, parent_key);

            let spec = match kind {
                models::consts::NONE => HallwaySpec::None,
                models::consts::ICE_PUZZLE => HallwaySpec::IcePuzzle,
                _ => return Err(Error::UnknownHallwayKind{ key, parent_key, kind }),
            };

            hallways_vec.push(spec);
        }

        // Interpret an empty hallways array as an array with a None kind in it.
        let hallways = hallways_vec.try_into().unwrap_or_default();

        Ok(Config {
            segments,
            entities,
            hallways,
        })
    }

    fn eval(code: &str) -> Result<Object, Error> {
        let context = init_context()?;
        let runtime = Arc::try_new(context.runtime()?)?;

        let mut sources = sources_with_helpers()?;
        sources.insert(Source::memory(code)?)?;

        let mut diagnostics = Diagnostics::new();

        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if has_meaningful_diagnostics(&diagnostics) {
            return Err(Error::Diagnostics(to_meaningful_diagnostics(diagnostics), sources))
        }

        let unit = result?;

        let mut vm = Vm::new(runtime, Arc::try_new(unit)?);

        let vm_output: rune::Value = vm.call(["main"], ())?;

        let nested_result: Result<Result<Object, String>, RuntimeError> = rune::from_value(vm_output);

        match nested_result {
            Ok(Ok(obj)) => Ok(obj),
            Ok(Err(e)) => Err(Error::FromConfig(e)),
            Err(e) => Err(Error::Runtime(e)),
        }


    }

    fn sources_with_helpers() -> Result<Sources, AllocError> {
        let mut sources = Sources::new();

        macro_rules! add_module {
            ($name: ident = $string: expr) => {{
                sources.insert(Source::new(
                    stringify!($name),
                    &format!(
                        // Have to have an explcit mod if we want to import them it seems.
                        // Don't add any extra lines, so that line numbers line up with the
                        // code that was passed in.
                        "mod {} {{ {} }}",
                        stringify!($name),
                        $string
                    ),
                )?)?;
            }};
        }

        let mut hallways_string = String::with_capacity(128);

        for (name, value) in models::consts::ALL_HALLWAY_KINDS {
            hallways_string += &format!("pub const {name} = {value};\n");
        }

        add_module!(hallways = hallways_string);

        let mut tile_flags_string = String::with_capacity(128);

        for (name, value) in models::consts::ALL_TILE_FLAGS {
            tile_flags_string += &format!("pub const {name} = {value};\n");
        }

        add_module!(tile_flags = tile_flags_string);

        let mut entity_flags_string = String::with_capacity(128);

        for (name, value) in models::consts::ALL_ENTITY_FLAGS {
            entity_flags_string += &format!("pub const {name} = {value};\n");
        }

        add_module!(entity_flags = entity_flags_string);

        let default_spritesheet_string = format!(r#"
            const TILES_PER_ROW = {TILES_PER_ROW};

            pub fn tile_sprite_xy(x, y) {{
                y * TILES_PER_ROW + x
            }}

            pub fn mob(n) {{
                tile_sprite_xy(3, n)
            }}

            pub fn item(n) {{
                tile_sprite_xy(4, n)
            }}
            // TODO: Define walls, floor, door animation, and player here too

            pub const OPEN_DOOR = 5 * TILES_PER_ROW + 0;//tile_sprite_xy(0, 5);
            pub const OPEN_END_DOOR = 5 * TILES_PER_ROW + 1;//tile_sprite_xy(1, 5);

            pub const DOOR_MATERIALS = ["gold", "iron", "carbon-steel"];
            pub const DOOR_COLOURS = ["red", "green", "blue"];

            // short for door and key xy.
            // Takes the door's xy, and assumes the key is one in the positive x direction.
            fn dak_xy(door_x, door_y) {{
                #{{
                    door: tile_sprite_xy(door_x, door_y),
                    key: tile_sprite_xy(door_x + 1, door_y)
                }}
            }}

            pub fn door_and_key_by_material_and_colour(material, colour) {{
                Ok(
                    match [material, colour] {{
                        ["gold", "red"] => dak_xy(0, 6),
                        ["gold", "green"] => dak_xy(0, 7),
                        ["gold", "blue"] => dak_xy(6, 0),
                        ["iron", "red"] => dak_xy(6, 1),
                        ["iron", "green"] => dak_xy(6, 2),
                        ["iron", "blue"] => dak_xy(6, 3),
                        ["carbon-steel", "red"] => dak_xy(6, 4),
                        ["carbon-steel", "green"] => dak_xy(6, 5),
                        ["carbon-steel", "blue"] => dak_xy(6, 6),
                        _ => return Err("No door and key found for \"" + material + "\"" + "and \"" + colour + "\""),
                    }}
                )
            }}
        "#);

        add_module!(default_spritesheet = default_spritesheet_string);

        let mut entity_ids_string = String::with_capacity(256);

        for (name, value) in models::consts::ALL_ENTITY_ID_REFERENCE_KINDS {
            entity_ids_string += &format!("pub const {name} = {value};\n");
        }

        entity_ids_string += r#"
            pub fn relative(n) {
                #{
                    kind: RELATIVE,
                    value: n,
                }
            }

            pub fn absolute(n) {
                #{
                    kind: ABSOLUTE,
                    value: n,
                }
            }
        "#;

        add_module!(entity_ids = entity_ids_string);

        let mut collect_actions_string = String::with_capacity(256);

        for (name, value) in models::consts::ALL_COLLECT_ACTION_KINDS {
            collect_actions_string += &format!("pub const {name} = {value};\n");
        }

        add_module!(collect_actions = collect_actions_string);

        Ok(sources)
    }

    /// Filter out unused warnings from the helper modules, in a hacky way.
    // TODO: Consider unused warnings outside of the helper moduels meaningful?
    //       This coudl be checked by knowing the source IDs for those modules
    fn is_meaningful(diagnostic: &Diagnostic, buffer: &mut String) -> bool {
        match diagnostic {
            Diagnostic::Warning(warning) => {
                use std::fmt::Write;
                buffer.clear();

                // This can't fail, unless maybe if we run out of memory.
                let _ = write!(buffer, "{warning:?}");

                // This is the hacky part, that we do because currently this info
                // isn't exposed another way. If the name changes we will start
                // seeing the warnings, which seems less bad than squelching all
                // warnings from the helper sources, and thus likely missing some
                // real ones.
                !buffer.contains("kind: NotUsed")
            },
            _ => {
                true
            }
        }
    }

    fn has_meaningful_diagnostics(diagnostics: &Diagnostics) -> bool {
        let mut buffer = String::with_capacity(128);

        for diagnostic in diagnostics.diagnostics() {
            if is_meaningful(diagnostic, &mut buffer) {
                return true;
            }
        }

        false
    }

    fn to_meaningful_diagnostics(diagnostics: Diagnostics) -> Vec<Diagnostic> {
        let diagnostics: rune::alloc::Vec<_> = diagnostics.into_diagnostics();

        // We don't want to expose the `rune::alloc::Vec` type outside this module.
        let mut output = Vec::with_capacity(diagnostics.len());
        let mut buffer = String::with_capacity(128);

        for d in diagnostics {
            if is_meaningful(&d, &mut buffer) {
                output.push(d);
            }
        }

        output
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn sources_with_helpers_builds() -> Result<(), Box<dyn std::error::Error>> {
            let context = init_context()?;
            let runtime = Arc::new(context.runtime()?);

            let mut sources = sources_with_helpers()?;

            let mut diagnostics = Diagnostics::new();

            let unit = rune::prepare(&mut sources)
                .with_context(&context)
                .with_diagnostics(&mut diagnostics)
                .build()
                .unwrap();


            if has_meaningful_diagnostics(&diagnostics) {
                use rune::termcolor::{ColorChoice, StandardStream};
                let mut writer = StandardStream::stderr(ColorChoice::Always);
                diagnostics.emit(&mut writer, &sources)?;
                assert!(diagnostics.is_empty(), "{diagnostics:#?}");
            }

            Ok(())
        }

        fn test_eval(code: &str) {
            match eval(code) {
                Err(e) => {
                    if let Error::Diagnostics(diagnostics, sources) = &e {
                        if !diagnostics.is_empty() {
                            assert!(diagnostics.is_empty(), "{diagnostics:#?}");
                        }
                    }

                    assert!(false, "should eval without errors: {e}");
                }
                Ok(_) => {}
            }
        }

        #[test]
        fn default_spritesheet_tests_pass() {
            let code = r#"
                use default_spritesheet as DS;

                pub fn main() {
                    for material in DS::DOOR_MATERIALS {
                        for colour in DS::DOOR_COLOURS {
                            let result = DS::door_and_key_by_material_and_colour(material, colour);

                            match result {
                                Ok(#{door, key}) => {
                                    if door < 0 {
                                        panic!("Negative door sprite for {material}, {colour}: {door}");
                                    }

                                    if key < 0 {
                                        panic!("Negative key sprite for {material}, {colour}: {key}");
                                    }
                                },
                                _ => {
                                    panic!("{result}");
                                }
                            }
                        }
                    }

                    Ok(#{})
                }
            "#;

            test_eval(code);
        }

        #[test]
        fn entity_flags_tests_pass() {
            let mut code = String::with_capacity(256);

            code += r#"
                use entity_flags as EF;

                pub fn main() {
                    let bits = 0;
            "#;

           for (name, value) in models::consts::ALL_ENTITY_FLAGS {
                // Just do something with all the flag values to prove they are there.
                code += &format!("bits |= EF::{name};\n");
            }

            code += r#"
                    Ok(#{})
                }
            "#;

            test_eval(&code);
        }

        #[test]
        fn to_config_works_on_a_small_config() {
            let code = r#"
                use hallways as HW;
                use tile_flags as TF;
                const A = TF::FLOOR | TF::ITEM_START | TF::NPC_START;

                pub fn main() {
                    Ok(#{
                        hallways: [
                            #{
                                kind: HW::NONE,
                            },
                        ],
                        entities: [
                            #{
                                flags: 0,
                                inventory_description: [
                                    ["Test enitity"]
                                ],
                                tile_sprite: 0,
                            },
                        ],
                        segments: [
                            #{
                                width: 1,
                                tiles: [
                                    A,
                                    A,
                                    A,
                                    A,
                                    A,
                                    A,
                                    A,
                                ],
                            },
                        ],
                    })
                }
            "#;

            let obj = eval(&code).expect("should eval properly");

            to_config(obj).expect("should extract config properly");
        }
    }

    fn init_context() -> Result<Context, ContextError> {
        let /* mut */ context = Context::with_default_modules()?;

        // TODO? Add native modules if we turn out to need any?

        Ok(context)
    }
}

mod hardcoded {
    use models::{Config};

    #[derive(Debug)]
    pub enum Error {

    }

    // Punting here because this code is temporary
    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{self:#?}")
        }
    }

    pub fn parse(_code: &str) -> Result<Config, Error> {
        use models::{*};
        use vec1::Vec1;
        // Hardcoding this to try out removing rhai, to see if that fixes bug that seems like UB
        // since miri reports UB in rhai.
        let config = Config {
            segments: Vec1::try_from(
                vec![
                    config::WorldSegment {
                        width: 14,
                        tiles: vec![
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            33,
                            33,
                            33,
                            33,
                            33,
                            33,
                            5,
                            0,
                            0,
                            5,
                            0,
                            0,
                            5,
                            5,
                            0,
                            0,
                            0,
                            0,
                            0,
                            33,
                            5,
                            0,
                            1,
                            25,
                            1,
                            0,
                            25,
                            5,
                            0,
                            9,
                            9,
                            9,
                            0,
                            33,
                            5,
                            5,
                            25,
                            0,
                            25,
                            5,
                            25,
                            5,
                            5,
                            1,
                            1,
                            1,
                            5,
                            33,
                            5,
                            0,
                            1,
                            25,
                            1,
                            0,
                            25,
                            5,
                            5,
                            5,
                            17,
                            5,
                            5,
                            33,
                            5,
                            0,
                            0,
                            9,
                            0,
                            0,
                            5,
                            5,
                            5,
                            17,
                            5,
                            17,
                            5,
                            33,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            5,
                            33,
                        ],
                    },
                    config::WorldSegment {
                        width: 7,
                        tiles: vec![
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            33,
                            5,
                            1,
                            0,
                            33,
                            0,
                            0,
                            0,
                            0,
                            1,
                            25,
                            5,
                            0,
                            0,
                            33,
                            1,
                            5,
                            25,
                            5,
                            0,
                            0,
                            0,
                            0,
                            1,
                            25,
                            5,
                            0,
                            0,
                            33,
                            5,
                            5,
                            0,
                            33,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                        ],
                    },
                    config::WorldSegment {
                        width: 7,
                        tiles: vec![
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            5,
                            1,
                            1,
                            1,
                            5,
                            0,
                            0,
                            5,
                            0,
                            0,
                            0,
                            5,
                            0,
                            0,
                            1,
                            33,
                            0,
                            33,
                            1,
                            0,
                            0,
                            1,
                            0,
                            33,
                            0,
                            1,
                            0,
                            0,
                            1,
                            1,
                            1,
                            1,
                            1,
                            0,
                            0,
                            25,
                            25,
                            25,
                            25,
                            25,
                            0,
                            0,
                            33,
                            5,
                            5,
                            5,
                            33,
                            0,
                            0,
                            25,
                            25,
                            25,
                            25,
                            25,
                            0,
                            0,
                            1,
                            1,
                            1,
                            1,
                            1,
                            0,
                            0,
                            1,
                            0,
                            33,
                            0,
                            1,
                            0,
                            0,
                            1,
                            33,
                            0,
                            33,
                            1,
                            0,
                            0,
                            5,
                            0,
                            0,
                            0,
                            5,
                            0,
                            0,
                            5,
                            1,
                            1,
                            1,
                            5,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                        ],
                    },
                ],
            ).unwrap(),
            entities: Vec1::try_from(
                vec![
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a chest, probably with something cool in it.".to_string(),
                                },
                                Speech {
                                    text: "can't seem to open it, so it'll stay at least probably\ncool forever.".to_string(),
                                },
                            ],
                        ],
                        id: 0,
                        flags: 3,
                        tile_sprite: 36,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "hey can you get me something that's at least probably\ncool?".to_string(),
                                },
                            ],
                            vec![
                                Speech {
                                    text: "a chest, for me? that's probably cool of you bro!".to_string(),
                                },
                                Speech {
                                    text: "i gotta be probably cool back. here have this thing i\nfound.".to_string(),
                                },
                            ],
                            vec![
                                Speech {
                                    text: "i am probably living the life with my probably cool\nthing in this chest!".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 1,
                        flags: 0,
                        tile_sprite: 35,
                        wants: vec![
                            0,
                        ],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "i lost my bayer-dollars! can you help me find them?".to_string(),
                                },
                                Speech {
                                    text: "i don't know where i lost them. i'm looking over here\nbecause the light is better.".to_string(),
                                },
                            ],
                            vec![
                                Speech {
                                    text: "you're giving me these bayer-dollars? i want them to\nbe mine, so they must be mine!".to_string(),
                                },
                                Speech {
                                    text: "i also want everyone to give rewards when people\nreturn stuff like this. so i have to too. here you go!".to_string(),
                                },
                            ],
                            vec![
                                Speech {
                                    text: "thanks for being the conduit to bring my\ndestined-for-me bayer dollars back!".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 2,
                        flags: 0,
                        tile_sprite: 43,
                        wants: vec![
                            3,
                        ],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "some bayer-dollars. you can tell because of the\npattern in the middle.".to_string(),
                                },
                            ],
                        ],
                        id: 3,
                        flags: 3,
                        tile_sprite: 44,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![],
                        id: 4,
                        flags: 30,
                        tile_sprite: 41,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![],
                        id: 5,
                        flags: 10,
                        tile_sprite: 40,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked red-gold door. bet the key is red-gold too.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 6,
                        flags: 8,
                        tile_sprite: 48,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a red-gold key. bet it opens a red-gold door.".to_string(),
                                },
                            ],
                        ],
                        id: 7,
                        flags: 3,
                        tile_sprite: 49,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 6,
                                    to: 4,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked green-gold door. bet the key is green-gold\ntoo.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 8,
                        flags: 8,
                        tile_sprite: 56,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a green-gold key. bet it opens a green-gold door.".to_string(),
                                },
                            ],
                        ],
                        id: 9,
                        flags: 3,
                        tile_sprite: 57,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 8,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked blue-gold door. bet the key is blue-gold too.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 10,
                        flags: 8,
                        tile_sprite: 6,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a blue-gold key. bet it opens a blue-gold door.".to_string(),
                                },
                            ],
                        ],
                        id: 11,
                        flags: 3,
                        tile_sprite: 7,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 10,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked red-iron door. bet the key is red-iron too.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 12,
                        flags: 8,
                        tile_sprite: 14,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a red-iron key. bet it opens a red-iron door.".to_string(),
                                },
                            ],
                        ],
                        id: 13,
                        flags: 3,
                        tile_sprite: 15,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 12,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked green-iron door. bet the key is green-iron\ntoo.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 14,
                        flags: 8,
                        tile_sprite: 22,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a green-iron key. bet it opens a green-iron door.".to_string(),
                                },
                            ],
                        ],
                        id: 15,
                        flags: 3,
                        tile_sprite: 23,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 14,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked blue-iron door. bet the key is blue-iron too.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 16,
                        flags: 8,
                        tile_sprite: 30,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a blue-iron key. bet it opens a blue-iron door.".to_string(),
                                },
                            ],
                        ],
                        id: 17,
                        flags: 3,
                        tile_sprite: 31,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 16,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked red-carbon-steel door. bet the key is\nred-carbon-steel too.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 18,
                        flags: 8,
                        tile_sprite: 38,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a red-carbon-steel key. bet it opens a\nred-carbon-steel door.".to_string(),
                                },
                            ],
                        ],
                        id: 19,
                        flags: 3,
                        tile_sprite: 39,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 18,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked green-carbon-steel door. bet the key is\ngreen-carbon-steel too.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 20,
                        flags: 8,
                        tile_sprite: 46,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a green-carbon-steel key. bet it opens a\ngreen-carbon-steel door.".to_string(),
                                },
                            ],
                        ],
                        id: 21,
                        flags: 3,
                        tile_sprite: 47,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 20,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                    EntityDef {
                        speeches: vec![
                            vec![
                                Speech {
                                    text: "a locked blue-carbon-steel door. bet the key is\nblue-carbon-steel too.".to_string(),
                                },
                            ],
                        ],
                        inventory_description: vec![],
                        id: 22,
                        flags: 8,
                        tile_sprite: 54,
                        wants: vec![],
                        on_collect: vec![],
                    },
                    EntityDef {
                        speeches: vec![],
                        inventory_description: vec![
                            vec![
                                Speech {
                                    text: "a blue-carbon-steel key. bet it opens a\nblue-carbon-steel door.".to_string(),
                                },
                            ],
                        ],
                        id: 23,
                        flags: 3,
                        tile_sprite: 55,
                        wants: vec![],
                        on_collect: vec![
                            CollectAction::Transform(
                                Transform {
                                    from: 22,
                                    to: 5,
                                },
                            ),
                        ],
                    },
                ],
            ).unwrap(),
            hallways: Vec1::try_from(
                vec![
                    models::config::HallwaySpec::None,
                    models::config::HallwaySpec::IcePuzzle,
                ],
            ).unwrap(),
        };

        Ok(config)
    }
}