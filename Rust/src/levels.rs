/// Campaign order from IronJump/Classes/EAGLView.m.
pub const CAMPAIGN_COUNT: usize = 30;
pub const LEVELS_PER_STAGE: usize = 11;

pub struct Level {
    pub name: String,
    pub xml: String,
    pub custom: bool,
}

pub fn bundled() -> Vec<Level> {
    macro_rules! level {
        ($name:literal, $custom:expr) => {
            Level {
                name: $name.into(),
                xml: include_str!(concat!(
                    "../../IronJump/XLevels/Converted/",
                    $name,
                    ".xlevel"
                ))
                .into(),
                custom: $custom,
            }
        };
    }
    vec![
        level!("tutorial", false),
        level!("easytrampoline", false),
        level!("trampoline", false),
        level!("exit", false),
        level!("platformslearn", false),
        level!("jump", false),
        level!("fun", false),
        level!("easyplatforms", false),
        level!("fall", false),
        level!("platforms", false),
        level!("closed", false),
        level!("trampoline2", false),
        level!("easyelevators", false),
        level!("elevators", false),
        level!("hardplatforms", false),
        level!("movablelearn", false),
        level!("movable", false),
        level!("puzzle", false),
        level!("elevatormadness", false),
        level!("magnets", false),
        level!("magnetshard", false),
        level!("magnetmoves", false),
        level!("speed", false),
        level!("moving1", false),
        level!("moving2", false),
        level!("speed2", false),
        level!("collisions", false),
        level!("speed3", false),
        level!("speed4", false),
        level!("labyrinth", false),
        // Archived development levels were not part of the iPhone campaign.
        level!("example", true),
        level!("test", true),
        level!("movable_test", true),
        level!("movableforjump", true),
        level!("trampognet", true),
    ]
}
