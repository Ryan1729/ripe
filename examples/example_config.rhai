import "tile_flags" as TF;
const W = TF::WALL;
const F = TF::FLOOR | TF::PLAYER_START;
const B = TF::FLOOR; // Bare Floor
const I = TF::FLOOR | TF::ITEM_START | TF::NPC_START;

import "entity_flags" as EF;

const MOB = 0;
const ITEM = EF::COLLECTABLE;

import "default_spritesheet" as DS;

#{
    segments: [
        #{
            width: 7,
            tiles: [
                F, F, F, F, F, F, F,
                F, W, W, F, W, W, F,
                F, W, B, I, B, W, F,
                F, F, I, W, I, F, F,
                F, W, B, I, B, W, F,
                F, W, W, I, W, W, F,
                F, F, F, F, F, F, F,
            ],
        },
    ],
    entities: [
        #{
            flags: MOB,
            speeches: [ "hey can you get me something that's at least probably cool?" ],
            tile_sprite: DS::mob(0),
        },
        #{
            flags: ITEM,
            inventory_description: [ "a chest, probably with something cool in it.", "can't seem to open it, so it'll stay at least probably cool forever." ],
            tile_sprite: DS::item(0),
        },
        #{
            flags: MOB,
            speeches: [ "I lost my bayer-dollars! Can you help me find them?", "I don't know where I lost them. I'm looking over here because the light is better." ],
            tile_sprite: DS::mob(1),
        },
        #{
            flags: ITEM,
            inventory_description: [ "some bayer-dollars. you can tell because of the pattern in the middle." ],
            tile_sprite: DS::item(1),
        },
    ],
}