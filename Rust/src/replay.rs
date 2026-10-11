//! Portable input traces for the bundled levels. Filesystem I/O belongs to the runner.
use crate::{
    input::{Action, FrameInput, GameInput},
    math::{Vec2, vec2},
};

pub struct Frame {
    pub elapsed: f64,
    pub viewport: Vec2,
    pub reset_clock: bool,
    pub input: FrameInput,
}

impl Frame {
    pub fn encode(&self) -> String {
        use std::fmt::Write;
        let mut line = format!(
            "F {} {} {} {} {} {}",
            self.elapsed,
            self.viewport.x,
            self.viewport.y,
            self.reset_clock,
            self.input.game.move_axis,
            self.input.game.jump
        );
        for action in &self.input.actions {
            let name = match action {
                Action::Quit => "quit",
                Action::Back => "back",
                Action::Confirm => "confirm",
                Action::Restart => "restart",
                Action::PreviousStage => "previous_stage",
                Action::NextStage => "next_stage",
                Action::PreviousLevel => "previous_level",
                Action::NextLevel => "next_level",
                Action::Click(point) => {
                    write!(line, " click:{}:{}", point.x, point.y).unwrap();
                    continue;
                }
            };
            write!(line, " {name}").unwrap();
        }
        line
    }

    pub fn parse(line: &str) -> Result<Self, String> {
        let mut fields = line
            .split('|')
            .next()
            .unwrap_or_default()
            .split_whitespace();
        if fields.next() != Some("F") {
            return Err("Expected trace frame".into());
        }
        fn field<T: std::str::FromStr>(
            fields: &mut std::str::SplitWhitespace<'_>,
        ) -> Result<T, String> {
            fields
                .next()
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| "Invalid trace field".into())
        }
        let elapsed: f64 = field(&mut fields)?;
        let viewport = vec2(field(&mut fields)?, field(&mut fields)?);
        let reset_clock = field(&mut fields)?;
        let move_axis: f32 = field(&mut fields)?;
        let jump = field(&mut fields)?;
        if !elapsed.is_finite()
            || elapsed < 0.0
            || !viewport.is_finite()
            || viewport.min_element() <= 0.0
            || !move_axis.is_finite()
            || move_axis.abs() > 1.0
        {
            return Err("Trace values outside valid range".into());
        }
        let mut actions = Vec::new();
        for name in fields {
            actions.push(match name {
                "quit" => Action::Quit,
                "back" => Action::Back,
                "confirm" => Action::Confirm,
                "restart" => Action::Restart,
                "previous_stage" => Action::PreviousStage,
                "next_stage" => Action::NextStage,
                "previous_level" => Action::PreviousLevel,
                "next_level" => Action::NextLevel,
                _ => {
                    let mut click = name.split(':');
                    if click.next() != Some("click") {
                        return Err("Unknown trace action".into());
                    }
                    let x = click
                        .next()
                        .and_then(|v| v.parse().ok())
                        .ok_or("Invalid click")?;
                    let y = click
                        .next()
                        .and_then(|v| v.parse().ok())
                        .ok_or("Invalid click")?;
                    let point = vec2(x, y);
                    if click.next().is_some() || !point.is_finite() {
                        return Err("Invalid click".into());
                    }
                    Action::Click(point)
                }
            });
        }
        Ok(Self {
            elapsed,
            viewport,
            reset_clock,
            input: FrameInput {
                game: GameInput::new(move_axis, jump),
                actions,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trace_preserves_timing_inputs_and_action_order() {
        let frame = Frame {
            elapsed: 0.0167123456789,
            viewport: vec2(700.25, 320.0),
            reset_clock: true,
            input: FrameInput {
                game: GameInput::new(-0.23456, true),
                actions: vec![
                    Action::NextStage,
                    Action::PreviousStage,
                    Action::NextLevel,
                    Action::PreviousLevel,
                    Action::Click(vec2(123.4, 250.0)),
                    Action::Confirm,
                    Action::Restart,
                    Action::Back,
                    Action::Quit,
                ],
            },
        };
        let decoded = Frame::parse(&format!("{} | 1 0.25 0.5", frame.encode())).unwrap();
        assert_eq!(decoded.elapsed, frame.elapsed);
        assert_eq!(decoded.viewport, frame.viewport);
        assert_eq!(decoded.reset_clock, frame.reset_clock);
        assert_eq!(decoded.input.game, frame.input.game);
        assert_eq!(decoded.input.actions, frame.input.actions);
        for invalid in [
            "F",
            "F NaN 700 320 false 0 false",
            "F 0.01 700 320 false 0 false unknown",
            "F 0.01 700 320 false 0 false click:NaN:0",
        ] {
            assert!(Frame::parse(invalid).is_err());
        }
    }
}
