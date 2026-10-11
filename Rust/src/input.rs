//! Platform adapters translate physical controls into these game inputs.
use crate::math::Vec2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInput {
    /// Analog steering in [-1, 1]: negative is left, positive is right.
    pub move_axis: f32,
    /// Holding jump repeats jumps whenever grounded, as in the original game.
    pub jump: bool,
}
impl GameInput {
    pub const NONE: Self = Self {
        move_axis: 0.0,
        jump: false,
    };
    pub const fn new(move_axis: f32, jump: bool) -> Self {
        Self { move_axis, jump }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Quit,
    Back,
    Confirm,
    Restart,
    PreviousStage,
    NextStage,
    PreviousLevel,
    NextLevel,
    /// A press in logical game coordinates, after excluding letterbox margins.
    Click(Vec2),
}

#[derive(Debug, Default)]
pub struct FrameInput {
    pub game: GameInput,
    /// Ordered, one-shot actions; never repeated for simulation catch-up ticks.
    pub actions: Vec<Action>,
}
