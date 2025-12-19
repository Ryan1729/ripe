import "tile_flags" as TF;
const A = TF::FLOOR | TF::ITEM_START | TF::NPC_START;
const B = TF::FLOOR; // Bare Floor
const D = TF::FLOOR | TF::DOOR_START;
const F = TF::FLOOR | TF::PLAYER_START;
const I = TF::FLOOR | TF::ITEM_START;
const N = TF::FLOOR | TF::NPC_START;
const W = TF::WALL;

import "entity_flags" as EF;

const MOB = 0;
const ITEM = EF::STEPPABLE | EF::COLLECTABLE;
const OPEN_DOOR = EF::DOOR | EF::STEPPABLE;
const END_DOOR = OPEN_DOOR | EF::VICTORY | EF::NOT_SPAWNED_AT_START;
const LOCKED_DOOR = EF::DOOR;

import "default_spritesheet" as DS;
import "entity_ids" as ID;
import "collect_actions" as CA;

let entities = [
    #{
        flags: ITEM,
        inventory_description: [
            ["a chest, probably with something cool in it.", "can't seem to open it, so it'll stay at least probably cool forever."]
        ],
        tile_sprite: DS::item(4),
    },
    #{
        flags: MOB,
        speeches: [
            ["hey can you get me something that's at least probably cool?"],
            ["A chest, for me? That's probably cool of you bro!", "I gotta be probably cool back. Here have this thing I found." ],
            ["I am probably living the life with my probably cool thing in this chest!"],
        ],
        tile_sprite: DS::mob(4),
        wants: [ID::relative(-1)],
    },
    #{
        flags: MOB,
        speeches: [
            [ "I lost my bayer-dollars! Can you help me find them?", "I don't know where I lost them. I'm looking over here because the light is better." ],
            [ "You're giving me these bayer-dollars? I want them to be mine, so they must be mine!", "I also want everyone to give rewards when people return stuff like this. So I have to too. Here you go!" ],
            [ "Thanks for being the conduit to bring my destined-for-me bayer dollars back!" ],
        ],
        tile_sprite: DS::mob(5),
        wants: [ID::relative(1)],
    },
    #{
        flags: ITEM,
        inventory_description: [
            ["some bayer-dollars. you can tell because of the pattern in the middle."],
        ],
        tile_sprite: DS::item(5),
    },
];

let open_end_door_id = ID::absolute(entities.len);
entities.push(
    #{
        flags: END_DOOR,
        tile_sprite: DS::OPEN_END_DOOR,
    },
);

let open_door_id = ID::absolute(entities.len);
entities.push(
    #{
        flags: OPEN_DOOR,
        tile_sprite: DS::OPEN_DOOR,
    },
);

const DOOR_MATERIALS = ["gold", "iron", "high-carbon steel"];
const DOOR_COLOURS = ["red", "green", "blue"];

for material in DS::DOOR_MATERIALS {
    for colour in DS::DOOR_COLOURS {
        let to = if material == DS::DOOR_MATERIALS[0] && colour == DS::DOOR_COLOURS[0] {
            open_end_door_id
        } else {
            open_door_id
        };

        let result = DS::door_and_key_by_material_and_colour(material, colour);

        if result.ok != () {
            entities.push(
                #{
                    flags: LOCKED_DOOR,
                    tile_sprite: result.ok.door,
                    speeches: [
                        [ `A locked {colour}-{material} door. Bet the key is {colour}-{material} too.` ],
                    ],
                },
            );
            entities.push(
                #{
                    flags: ITEM,
                    tile_sprite: result.ok.key,
                    inventory_description: [
                        [`A {colour}-{material} key. Bet it opens a {colour}-{material} door.`],
                    ],
                    on_collect: [
                        #{
                            kind: CA::TRANSFORM,
                            from: ID::relative(-1),
                            to: to,
                        }
                    ],
                }
            );
        } else {
            throw result.err
        }
    }
}

#{
    segments: [
        #{
            width: 14,
            tiles: [
                F, F, F, F, F, F, F, F, D, D, D, D, D, D,
                F, W, W, F, W, W, F, F, W, W, W, W, W, D,
                F, W, B, A, B, W, F, F, W, I, I, I, W, D,
                F, F, A, W, A, F, F, F, F, B, B, B, F, D,
                F, W, B, A, B, W, F, F, F, F, N, F, F, D,
                F, W, W, I, W, W, F, F, F, N, F, N, F, D,
                F, F, F, F, F, F, F, F, F, F, F, F, F, D,
            ],
        },
        #{
            width: 7,
            tiles: [
                W, W, W, W, W, W, W,
                W, D, F, B, W, D, W,
                W, W, W, B, A, F, W,
                W, D, B, F, A, F, W,
                W, W, W, B, A, F, W,
                W, D, F, F, W, D, W,
                W, W, W, W, W, W, W,
            ],
        },
    ],
    entities: entities,
}