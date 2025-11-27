import "tile_flags" as TF;
const W = TF::WALL;
const F = TF::FLOOR | TF::PLAYER_START;
const B = TF::FLOOR; // Bare Floor
const I = TF::FLOOR | TF::ITEM_START;
const N = TF::FLOOR | TF::NPC_START;
const A = TF::FLOOR | TF::ITEM_START | TF::NPC_START;

import "entity_flags" as EF;

const MOB = 0;
const ITEM = EF::COLLECTABLE;

import "default_spritesheet" as DS;
import "entity_ids" as ID;

#{
    segments: [
        #{
            width: 14,
            tiles: [
                F, F, F, F, F, F, F, F, F, F, F, F, F, F,
                F, W, W, F, W, W, F, F, W, W, W, W, W, F,
                F, W, B, A, B, W, F, F, W, I, I, I, W, F,
                F, F, A, W, A, F, F, F, F, B, B, B, F, F,
                F, W, B, A, B, W, F, F, F, F, N, F, F, F,
                F, W, W, I, W, W, F, F, F, N, F, N, F, F,
                F, F, F, F, F, F, F, F, F, F, F, F, F, F,
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
                ["hey can you get me something that's at least probably cool?" ],
                ["A chest, for me? That's probably cool of you bro!", "I gotta be probably cool back. Here have this thing I found." ],
            ],
            tile_sprite: DS::mob(0),
            wants: [ID::relative(-1)],
        },
        #{
            flags: MOB,
            speeches: [
                [ "I lost my bayer-dollars! Can you help me find them?", "I don't know where I lost them. I'm looking over here because the light is better." ],
                [ "You're giving me these bayer-dollars? I want them to be mine, so they must be mine!", "I also want everyone to give rewards when people return stuff like this. So I have to too. Here you go!" ],
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
    ],
}