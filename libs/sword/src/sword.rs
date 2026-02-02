use gfx::{Commands};
use platform_types::{sprite, Input, Speaker};
use xs::Xs;

use std::collections::BTreeMap;

// Sketching this out, this is seeming a lot like the main game parts, which makes sense, 
// because the intended game has te same grid based movement etc.
// But I am reluctant to actually make them depend on too many of the same things, since
// changing how one of them works shouldn't affect the other.
// On the other hand, it doesn't seem liek a trivial amount of code to do a bunch of stuff
// that I think both of them will both do for the forseeable future. (again, with a risk of
// partial divergence down the line)
// Thus, I am thinking it makes sense to copy some code into here from the main game parts.
// But, before I do that, I think it's worthwhile to take some time try to simplify and 
// reduce the amount of unneeded lines of that code in-situ, so there's less that there are
// duplicate versions of. More generally, it's worth considering completing any TODOs in the
// to-be-duplicated code before the copy, as well.

pub mod xy {
    type Inner = u8;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct X(pub Inner);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Y(pub Inner);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }
}
use xy::XY;

type TileSprite = u8;

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub xy: XY,
    pub tile_sprite: TileSprite,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Key {
    pub xy: XY,
}

pub type Entities = BTreeMap<Key, Entity>;

#[derive(Clone, Debug)]
pub struct State {
    pub rng: Xs,
    pub player: Entity,
    pub mobs: Entities,
}

impl State {
    pub fn new(rng: &mut Xs) -> Self {
        let seed = xs::new_seed(rng);

        let mut rng = xs::from_seed(seed);

        let player = Entity::default();

        Self {
            rng,
            player,
            mobs: Entities::default(),
        }
    }

    pub fn is_complete(&self) -> bool {
        true
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        spec: &sprite::Spec::<sprite::SWORD>,
        input: Input,
        speaker: &mut Speaker,
    ) {
        //for entity in self.all_entities() {
            //commands.sspr(
//
            //)
        //}
    }
}
