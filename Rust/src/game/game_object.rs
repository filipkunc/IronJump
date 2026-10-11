use super::{elevator::Elevator, player::Player};
use crate::assets::Assets;
use macroquad::prelude::*;

pub enum WorldEffect {
    BounceMovables { surface: Rect, speed: f32 },
}

pub trait GameObject {
    fn update(&mut self, context: &UpdateContext<'_>);
    fn draw(&self, assets: &Assets);
    fn move_world(&mut self, offset: Vec2);
    fn rect(&self) -> Rect;

    fn interact(&mut self, _player: &mut Player) -> Option<WorldEffect> {
        None
    }
    fn set_vertical_speed(&mut self, _speed: f32) {}
    fn is_visible(&self) -> bool {
        true
    }
    fn is_transparent(&self) -> bool {
        true
    }
    fn is_diamond(&self) -> bool {
        false
    }
    fn is_exit(&self) -> bool {
        false
    }

    fn is_platform(&self) -> bool {
        false
    }

    fn is_movable(&self) -> bool {
        false
    }

    fn as_elevator(&self) -> Option<&Elevator> {
        None
    }

    fn as_elevator_mut(&mut self) -> Option<&mut Elevator> {
        None
    }
}

// Read-only access to the player and every object except the one being updated.
pub struct UpdateContext<'a> {
    pub(super) player_rect: Rect,
    pub(super) before: &'a [Box<dyn GameObject>],
    pub(super) after: &'a [Box<dyn GameObject>],
}

impl UpdateContext<'_> {
    pub(super) fn platform_rects(&self) -> impl Iterator<Item = Rect> + '_ {
        self.before
            .iter()
            .chain(self.after.iter())
            .filter(|object| object.is_platform())
            .map(|object| object.rect())
    }
}
