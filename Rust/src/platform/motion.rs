//! Sensor-to-game mapping, with no JNI, Android, or rendering dependencies.
use ironjump::input::GameInput;

const GRAVITY: f32 = 9.80665;
const STEERING_DEAD_ZONE: f32 = 0.02;
const JUMP_THRESHOLD: f32 = 0.05;
const STALE_AFTER: f64 = 0.25;
// Keep a short gesture visible across sensor/render/simulation rate differences.
const JUMP_HOLD: f64 = 0.1;

#[derive(Default)]
pub(super) struct MotionInput {
    last_sample: Option<f64>,
    rotation: i32,
    jump_baseline: f32,
    move_axis: f32,
    jump_until: f64,
}

impl MotionInput {
    /// Android accelerometer values are m/s² in the device's natural orientation.
    /// Rotation is Surface.ROTATION_0/90/180/270 (0/1/2/3); time is monotonic seconds.
    pub fn sample(&mut self, x: f32, y: f32, rotation: i32, time: f64) {
        if !x.is_finite() || !y.is_finite() || !time.is_finite() || time < 0.0 {
            return;
        }
        if self.last_sample.is_some_and(|previous| time <= previous) {
            return;
        }
        let (screen_x, screen_y) = match rotation {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, -y),
            3 => (y, -x),
            _ => return,
        };
        // Android reports proper acceleration, opposite the original iOS gravity
        // convention. A lower right edge must steer right in either landscape.
        let tilt = -screen_x / GRAVITY;
        self.move_axis = if tilt.abs() < STEERING_DEAD_ZONE {
            0.0
        } else {
            tilt.clamp(-1.0, 1.0)
        };
        let vertical = -screen_y / GRAVITY;
        match self.last_sample {
            Some(previous) if rotation == self.rotation && time - previous <= STALE_AFTER => {
                if vertical - self.jump_baseline > JUMP_THRESHOLD {
                    self.jump_until = time + JUMP_HOLD;
                }
                // EAGLView used lerp(current, previous, 0.4) at 30 Hz. Use the
                // equivalent decay for the actual sensor interval on this phone.
                let decay = 0.4_f32.powf(((time - previous) * 30.0) as f32);
                self.jump_baseline = vertical * (1.0 - decay) + self.jump_baseline * decay;
            }
            _ => {
                // Seed from the current pose; starting/resuming/rotating isn't a jump.
                self.jump_baseline = vertical;
                self.jump_until = 0.0;
            }
        }
        self.last_sample = Some(time);
        self.rotation = rotation;
    }

    pub fn input(&self, time: f64) -> GameInput {
        let Some(previous) = self.last_sample else {
            return GameInput::NONE;
        };
        if !time.is_finite() || time < previous || time - previous > STALE_AFTER {
            return GameInput::NONE;
        }
        GameInput::new(self.move_axis, time < self.jump_until)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilt_direction_and_strength_follow_the_screen_in_all_orientations() {
        // Same screen-space acceleration: right edge lowered, screen held upright.
        for (rotation, x, y) in [
            (0, -0.3, 0.8),
            (1, 0.8, 0.3),
            (2, 0.3, -0.8),
            (3, -0.8, -0.3),
        ] {
            let mut motion = MotionInput::default();
            motion.sample(x * GRAVITY, y * GRAVITY, rotation, 1.0);
            assert!((motion.input(1.0).move_axis - 0.3).abs() < 0.00001);
            assert!(!motion.input(1.0).jump);
            motion.sample(-x * GRAVITY, -y * GRAVITY, rotation, 1.1);
            assert!((motion.input(1.1).move_axis + 0.3).abs() < 0.00001);
        }
    }

    #[test]
    fn steering_ignores_small_tilts_and_clamps_extreme_acceleration() {
        let mut motion = MotionInput::default();
        motion.sample(0.0, 0.019 * GRAVITY, 1, 1.0);
        assert_eq!(motion.input(1.0), GameInput::NONE);
        motion.sample(0.0, 2.0 * GRAVITY, 1, 1.1);
        assert_eq!(motion.input(1.1).move_axis, 1.0);
        motion.sample(0.0, -2.0 * GRAVITY, 1, 1.2);
        assert_eq!(motion.input(1.2).move_axis, -1.0);
    }

    #[test]
    fn jump_detects_a_vertical_gesture_but_not_a_steady_pose() {
        for (rotation, sign) in [(1, 1.0), (3, -1.0)] {
            let mut motion = MotionInput::default();
            motion.sample(sign * GRAVITY, 0.0, rotation, 1.0);
            assert!(!motion.input(1.0).jump);
            motion.sample(sign * GRAVITY, 0.0, rotation, 1.1);
            assert!(!motion.input(1.1).jump);
            motion.sample(sign * 0.9 * GRAVITY, 0.0, rotation, 1.2);
            assert!(motion.input(1.2).jump);
            assert!(motion.input(1.25).jump); // A short pulse survives a render frame.
            assert!(!motion.input(1.31).jump);
        }
    }

    #[test]
    fn jump_filter_matches_original_at_30_hz_and_adapts_to_faster_sensors() {
        let mut slow = MotionInput::default();
        let mut fast = MotionInput::default();
        for motion in [&mut slow, &mut fast] {
            motion.sample(GRAVITY, 0.0, 1, 1.0);
        }
        slow.sample(0.5 * GRAVITY, 0.0, 1, 1.0 + 1.0 / 30.0);
        fast.sample(0.5 * GRAVITY, 0.0, 1, 1.0 + 1.0 / 60.0);
        fast.sample(0.5 * GRAVITY, 0.0, 1, 1.0 + 1.0 / 30.0);
        assert!((slow.jump_baseline - (-0.5 * 0.6 - 0.4)).abs() < 0.00001);
        assert!((slow.jump_baseline - fast.jump_baseline).abs() < 0.00001);
    }

    #[test]
    fn rotation_and_gaps_reset_jump_and_stale_samples_release_input() {
        let mut motion = MotionInput::default();
        motion.sample(GRAVITY, GRAVITY, 1, 1.0);
        assert_eq!(motion.input(1.26), GameInput::NONE);
        motion.sample(-GRAVITY, -GRAVITY, 3, 1.1);
        assert!(!motion.input(1.1).jump);
        motion.sample(-0.5 * GRAVITY, -GRAVITY, 3, 2.0);
        assert!(!motion.input(2.0).jump);
        assert_eq!(MotionInput::default().input(2.0), GameInput::NONE);
    }

    #[test]
    fn invalid_or_out_of_order_samples_do_not_corrupt_the_filter() {
        let mut motion = MotionInput::default();
        motion.sample(GRAVITY, 0.3 * GRAVITY, 1, 1.0);
        let expected = motion.input(1.0);
        motion.sample(0.0, f32::NAN, 1, 1.1);
        motion.sample(f32::INFINITY, 0.0, 1, 1.1);
        motion.sample(0.0, 0.0, 4, 1.1);
        motion.sample(0.0, 0.0, 1, f64::NAN);
        motion.sample(0.0, 0.0, 1, 0.9);
        assert_eq!(motion.input(1.0), expected);
    }
}
