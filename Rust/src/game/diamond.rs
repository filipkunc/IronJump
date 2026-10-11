use super::{
    collision::intersects,
    game_object::{GameObject, UpdateContext, WorldEffect},
    level_loader::parse_value,
    player::Player,
};
use crate::assets::Assets;
use macroquad::prelude::*;

pub(super) struct Diamond {
    position: Vec2,
    visible: bool,
}

impl Diamond {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            visible: true,
        }
    }
}

impl GameObject for Diamond {
    fn update(&mut self, _context: &UpdateContext<'_>) {}

    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        if self.visible && intersects(self.rect(), player.rect()) {
            self.visible = false;
        }
        None
    }

    fn draw(&self, assets: &Assets) {
        assets.diamond.draw(self.position.x, self.position.y);
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }

    fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 32.0, 32.0)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn is_diamond(&self) -> bool {
        true
    }
}
