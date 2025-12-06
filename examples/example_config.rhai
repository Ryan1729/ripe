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
const END_DOOR = EF::STEPPABLE | EF::VICTORY | EF::NOT_SPAWNED_AT_START;

import "default_spritesheet" as DS;
import "entity_ids" as ID;
import "collect_actions" as CA;

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
    ],
    entities: [
        #{
            flags: ITEM,
            inventory_description: [
                ["a chest, probably with something cool in it.", "can't seem to open it, so it'll stay at least probably cool forever."]
            ],
            tile_sprite: DS::item(0),
        },
        #{
            flags: MOB,
            speeches: [
                ["hey can you get me something that's at least probably cool?"],
                ["A chest, for me? That's probably cool of you bro!", "I gotta be probably cool back. Here have this thing I found." ],
                ["I am probably living the life with my probably cool thing in this chest!"],
            ],
            tile_sprite: DS::mob(0),
            wants: [ID::relative(-1)],
        },
        #{
            flags: MOB,
            speeches: [
                [ "I lost my bayer-dollars! Can you help me find them?", "I don't know where I lost them. I'm looking over here because the light is better." ],
                [ "You're giving me these bayer-dollars? I want them to be mine, so they must be mine!", "I also want everyone to give rewards when people return stuff like this. So I have to too. Here you go!" ],
                [ "Thanks for being the conduit to bring my destined-for-me bayer dollars back!" ],
            ],
            tile_sprite: DS::mob(1),
            wants: [ID::relative(1)],
        },
        #{
            flags: ITEM,
            inventory_description: [
                ["some bayer-dollars. you can tell because of the pattern in the middle."],
            ],
            tile_sprite: DS::item(1),
        },
        #{
            flags: END_DOOR,
            tile_sprite: DS::OPEN_DOOR,
        },
        #{
            flags: MOB,
            tile_sprite: DS::LOCKED_DOOR_1,
        },
        #{
            flags: ITEM,
            tile_sprite: DS::KEY_1,
            on_collect: [
                #{
                    kind: CA::TRANSFORM,
                    from: ID::relative(-1),
                    to: ID::relative(-2),
                }
            ],
        },
    ],
}