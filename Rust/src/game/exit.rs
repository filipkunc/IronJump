use super::{
    collision::intersects,
    game_object::{GameObject, UpdateContext, WorldEffect},
    level_loader::parse_value,
    player::Player,
};
use crate::assets::Assets;
use macroquad::prelude::*;

pub(super) struct Exit {
    position: Vec2,
    visible: bool,
}

impl Exit {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            visible: true,
        }
    }
}

impl GameObject for Exit {
    fn update(&mut self, _context: &UpdateContext<'_>) {}

    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        if self.visible && intersects(self.rect(), player.rect()) {
            self.visible = false;
        }
        None
    }

    fn draw(&self, assets: &Assets) {
        assets.exit.draw(self.position.x, self.position.y);
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }

    fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 64.0, 64.0)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn is_exit(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::game_with;

    #[test]
    fn diamonds_are_collected_once_and_exit_does_not_require_all_diamonds() {
        let mut game = game_with(
            "<FPDiamond><x>0</x><y>0</y></FPDiamond><FPDiamond><x>500</x><y>0</y></FPDiamond><FPExit><x>0</x><y>0</y></FPExit>",
        );
        assert_eq!(game.diamonds(), (0, 2));
        assert!(!game.is_complete());
        game.world.update(&mut game.player);
        assert_eq!(game.diamonds(), (1, 2));
        assert!(game.is_complete());
        game.world.update(&mut game.player);
        assert_eq!(game.diamonds(), (1, 2));
        assert_eq!(game.world.game_objects.len(), 3); // Stable elevator indices.
    }
}
