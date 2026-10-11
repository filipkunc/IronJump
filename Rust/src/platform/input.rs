use ironjump::{
    input::{Action, FrameInput, GameInput},
    viewport::Viewport,
};
use macroquad::prelude::*;

pub const CONTROL_HINT: &str = "Arrows / A,D: move   Space: jump   R: restart   Esc: menu";
pub const RETURN_TO_MENU_HINT: &str = "Enter: menu";

pub fn poll(view: Viewport) -> FrameInput {
    let mut actions = Vec::new();
    for (key, action) in [
        (KeyCode::Q, Action::Quit),
        (KeyCode::Escape, Action::Back),
        (KeyCode::Left, Action::PreviousStage),
        (KeyCode::Right, Action::NextStage),
        (KeyCode::Up, Action::PreviousLevel),
        (KeyCode::Down, Action::NextLevel),
        (KeyCode::Enter, Action::Confirm),
        (KeyCode::R, Action::Restart),
    ] {
        if is_key_pressed(key) {
            actions.push(action);
        }
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(point) = view.game_position(Vec2::from(mouse_position()))
    {
        actions.push(Action::Click(point));
    }
    let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    let mut game = GameInput::new(
        if left {
            -1.0
        } else if right {
            1.0
        } else {
            0.0
        },
        is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space),
    );
    // Touch is translated to the same logical input as keyboard or future sensors.
    for touch in touches() {
        if touch.phase == TouchPhase::Ended || touch.phase == TouchPhase::Cancelled {
            continue;
        }
        let Some(point) = view.game_position(touch.position) else {
            continue;
        };
        if point.x > view.logical_size.x * (2.0 / 3.0) {
            game.jump = true;
        } else {
            game.move_axis = if point.x < view.logical_size.x / 3.0 {
                -1.0
            } else {
                1.0
            };
        }
    }
    FrameInput { game, actions }
}
