//! Application flow and fixed simulation timing, shared by all platform runners.
use crate::{
    input::{Action, FrameInput},
    session::{Mode, Session},
};

const STEP: f64 = 1.0 / 60.0;

pub struct App {
    pub session: Session,
    accumulator: f64,
}
impl App {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            accumulator: 0.0,
        }
    }

    /// Discard accumulated time. A platform resuming from suspension should also
    /// restart its elapsed-time measurement before submitting the next frame.
    pub fn reset_clock(&mut self) {
        self.accumulator = 0.0;
    }

    /// Returns false when the platform should close the application.
    pub fn frame(&mut self, input: &FrameInput, elapsed: f64) -> bool {
        for &action in &input.actions {
            if !self.action(action) {
                return false;
            }
        }
        if elapsed.is_finite() && elapsed > 0.0 {
            self.accumulator += elapsed.min(0.25);
        }
        while self.accumulator >= STEP {
            self.session.update(input.game);
            self.accumulator -= STEP;
        }
        true
    }

    fn action(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return false,
            Action::Back => {
                if self.session.mode == Mode::Menu {
                    return false;
                }
                self.session.menu();
                self.reset_clock();
            }
            Action::Click(point) => self.session.click(point),
            _ => match self.session.mode {
                Mode::Menu => match action {
                    Action::PreviousStage => self.session.change_page(-1),
                    Action::NextStage => self.session.change_page(1),
                    Action::PreviousLevel => self
                        .session
                        .select(self.session.progress.selected.saturating_sub(1)),
                    Action::NextLevel => self.session.select(self.session.progress.selected + 1),
                    Action::Confirm => self.session.start(),
                    _ => {}
                },
                Mode::Victory => {
                    if action == Action::Confirm {
                        self.session.menu();
                    }
                }
                Mode::Playing => {
                    if action == Action::Restart {
                        self.session.start();
                    }
                }
            },
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{input::GameInput, progress::Progress};

    fn app() -> App {
        App::new(Session::new(Progress::default(), Vec::new()))
    }

    #[test]
    fn actions_are_processed_once_even_when_frame_runs_multiple_ticks() {
        let mut app = app();
        let input = FrameInput {
            actions: vec![Action::NextStage],
            ..Default::default()
        };
        assert!(app.frame(&input, 0.2));
        assert_eq!(app.session.page, 1);
        app.frame(&FrameInput::default(), 0.2);
        assert_eq!(app.session.page, 1);
    }

    #[test]
    fn back_returns_to_menu_then_quits_and_confirm_starts() {
        let mut app = app();
        assert!(app.frame(
            &FrameInput {
                actions: vec![Action::Confirm],
                ..Default::default()
            },
            0.0
        ));
        assert_eq!(app.session.mode, Mode::Playing);
        let back = FrameInput {
            actions: vec![Action::Back],
            ..Default::default()
        };
        assert!(app.frame(&back, 0.0));
        assert_eq!(app.session.mode, Mode::Menu);
        assert!(!app.frame(&back, 0.0));
    }

    #[test]
    fn clock_accumulates_fractional_ticks_and_discards_paused_time() {
        let mut app = app();
        let input = FrameInput {
            game: GameInput::NONE,
            ..Default::default()
        };
        app.frame(&input, STEP * 0.4);
        assert!((app.accumulator - STEP * 0.4).abs() < 1e-10);
        app.frame(&input, STEP * 0.7);
        assert!((app.accumulator - STEP * 0.1).abs() < 1e-10);
        app.reset_clock();
        assert_eq!(app.accumulator, 0.0);
        app.frame(&input, f64::NAN);
        assert_eq!(app.accumulator, 0.0);
    }

    #[test]
    fn simulation_is_independent_of_render_frame_rate() {
        use crate::drawing::test_support::RecordingRenderer;
        let mut normal = app();
        let mut faster = app();
        normal.session.start();
        faster.session.start();
        let input = FrameInput {
            game: GameInput::new(0.5, true),
            ..Default::default()
        };
        for _ in 0..60 {
            normal.frame(&input, STEP);
            faster.frame(&input, STEP * 0.5);
            faster.frame(&input, STEP * 0.5);
        }
        let mut normal_frame = RecordingRenderer::default();
        let mut faster_frame = RecordingRenderer::default();
        normal.session.draw(&mut normal_frame, "Menu");
        faster.session.draw(&mut faster_frame, "Menu");
        assert_eq!(normal_frame.commands, faster_frame.commands);
    }
}
