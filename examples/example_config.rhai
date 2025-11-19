import "tile_flags" as TF;
const W = TF::WALL;
const F = TF::FLOOR | TF::PLAYER_START;
const B = TF::FLOOR; // Bare Floor
const I = TF::FLOOR | TF::ITEM_START | TF::NPC_START;

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
        
    ],
}