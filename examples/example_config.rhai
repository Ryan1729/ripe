import "tile_flags" as TF;
const W = TF::WALL;
const F = TF::FLOOR | TF::PLAYER_START;
const B = TF::FLOOR; // Bare Floor
const I = TF::FLOOR | TF::ITEM_START | TF::NPC_START;

import "entity_defs" as ED;

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
            kind: ED::MOB,
            speeches: [ "hey can you get me a specific item?" ],
        },
        #{
            kind: ED::ITEM,
            speeches: [ "hey can you get me a specific item?" ],
        },
    ],
}