use crate::drawing::{
    BLACK, Color, GAME_HEIGHT, GAME_WIDTH, RED, Renderer, SpriteId, SpriteStyle, WHITE,
};
use crate::input::GameInput;
use crate::math::*;
use crate::progress::Progress;
use crate::{
    game::Game,
    levels::{self, CAMPAIGN_COUNT, LEVELS_PER_STAGE, Level},
};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Menu,
    Playing,
    Victory,
}

pub struct Session {
    pub levels: Vec<Level>,
    pub progress: Progress,
    pub mode: Mode,
    pub game: Option<Game>,
    pub page: usize,
    pub dirty: bool,
    pub error: Option<String>,
    pages: Vec<Range<usize>>,
    complete_ticks: u32,
    fallen_ticks: u32,
    win_animation: f32,
    viewport_size: Vec2,
}

impl Session {
    pub fn new(progress: Progress, additional: Vec<Level>) -> Self {
        let mut levels = levels::bundled();
        levels.extend(additional);
        let mut pages = Vec::new();
        for range in [0..CAMPAIGN_COUNT, CAMPAIGN_COUNT..levels.len()] {
            for start in range.clone().step_by(LEVELS_PER_STAGE) {
                pages.push(start..(start + LEVELS_PER_STAGE).min(range.end));
            }
        }
        let mut progress = progress;
        progress.unlocked = progress.unlocked.min(CAMPAIGN_COUNT - 1);
        progress.selected = progress.selected.min(levels.len() - 1);
        if progress.selected < CAMPAIGN_COUNT {
            progress.selected = progress.selected.min(progress.unlocked);
        }
        let page = pages
            .iter()
            .position(|range| range.contains(&progress.selected))
            .unwrap_or(0);
        Self {
            levels,
            progress,
            mode: Mode::Menu,
            game: None,
            page,
            pages,
            dirty: false,
            complete_ticks: 0,
            fallen_ticks: 0,
            win_animation: 0.0,
            viewport_size: vec2(GAME_WIDTH, GAME_HEIGHT),
            error: None,
        }
    }

    pub fn available(&self, level: usize) -> bool {
        level < self.levels.len() && (self.levels[level].custom || level <= self.progress.unlocked)
    }

    pub fn select(&mut self, level: usize) {
        if self.available(level) {
            self.progress.selected = level;
            self.dirty = true;
            self.page = self
                .pages
                .iter()
                .position(|range| range.contains(&level))
                .unwrap();
        }
    }

    pub fn resize(&mut self, size: Vec2) {
        if !size.is_finite() || size.min_element() <= 0.0 {
            return;
        }
        self.viewport_size = size;
        if let Some(game) = &mut self.game {
            game.resize(size);
        }
    }

    pub fn start(&mut self) {
        let mut game = Game::with_viewport(self.viewport_size);
        match game.load(&self.levels[self.progress.selected].xml) {
            Ok(()) => {
                self.game = Some(game);
                self.mode = Mode::Playing;
                self.complete_ticks = 0;
                self.fallen_ticks = 0;
                self.win_animation = 0.0;
                self.error = None;
                self.dirty = true;
            }
            Err(error) => {
                self.error = Some(error);
                self.mode = Mode::Menu;
                self.game = None;
            }
        }
    }

    pub fn menu(&mut self) {
        self.mode = Mode::Menu;
        self.game = None;
    }

    fn next_level(&mut self) {
        let next = self.progress.selected + 1;
        if next == CAMPAIGN_COUNT || next == self.levels.len() {
            self.mode = Mode::Victory;
            self.win_animation = 0.0;
        } else {
            if next < CAMPAIGN_COUNT {
                self.progress.unlocked = self.progress.unlocked.max(next);
            }
            self.select(next);
            self.start();
        }
    }

    pub fn update(&mut self, input: GameInput) {
        if self.mode == Mode::Victory {
            self.win_animation = (self.win_animation + 0.0015).min(0.7);
            return;
        }
        if self.mode != Mode::Playing {
            return;
        }
        let game = self.game.as_mut().unwrap();
        self.complete_ticks = if game.is_complete() {
            self.complete_ticks + 1
        } else {
            0
        };
        self.fallen_ticks = if game.has_fallen() {
            self.fallen_ticks + 1
        } else {
            0
        };
        if self.complete_ticks > 30 {
            self.next_level();
            return;
        }
        if self.fallen_ticks > 30 {
            self.start();
            return;
        }
        game.update(input);
    }

    pub fn change_page(&mut self, direction: i32) {
        self.page = (self.page as i32 + direction).clamp(0, self.pages.len() as i32 - 1) as usize;
    }

    pub fn click(&mut self, point: Vec2) {
        if self.mode == Mode::Victory {
            self.menu();
            return;
        }
        if self.mode != Mode::Menu {
            return;
        }
        if Rect::new(8.0, 102.0, 55.0, 112.0).contains(point) {
            self.change_page(-1);
            return;
        }
        if Rect::new(418.0, 102.0, 55.0, 112.0).contains(point) {
            self.change_page(1);
            return;
        }
        for (slot, level) in self.pages[self.page].clone().enumerate() {
            let position = level_position(slot);
            if Rect::new(position.x, position.y + 10.0, 44.0, 44.0).contains(point) {
                self.select(level);
                return;
            }
        }
        if self.pages[self.page].contains(&self.progress.selected)
            && Rect::new(160.0, 240.0, 164.0, 61.0).contains(point)
        {
            self.start();
        }
    }

    pub fn draw(&self, renderer: &mut dyn Renderer, return_to_menu_hint: &str) {
        self.draw_interpolated(renderer, return_to_menu_hint, 1.0);
    }

    pub fn draw_interpolated(
        &self,
        renderer: &mut dyn Renderer,
        return_to_menu_hint: &str,
        alpha: f32,
    ) {
        if let Some(game) = &self.game {
            game.draw_interpolated(
                renderer,
                self.levels[self.progress.selected].custom,
                if self.mode == Mode::Playing {
                    alpha
                } else {
                    1.0
                },
            );
            if self.mode == Mode::Victory {
                renderer.rectangle(
                    Rect::new(0.0, 0.0, self.viewport_size.x, self.viewport_size.y),
                    Color::new(0.0, 0.0, 0.0, self.win_animation),
                );
                let left = self.viewport_size.x * 0.5 - 128.0;
                let y = self.viewport_size.y
                    - (self.win_animation / 0.7) * (self.viewport_size.y * 0.5 + 36.0);
                renderer.sprite(SpriteId::Win, vec2(left, y));
                for x in 0..8 {
                    renderer.sprite(
                        SpriteId::Elevator(2),
                        vec2(left + x as f32 * 32.0, y + 64.0),
                    );
                }
                renderer.bitmap_text(
                    return_to_menu_hint,
                    vec2(
                        self.viewport_size.x * 0.5 - 80.0,
                        self.viewport_size.y - 45.0,
                    ),
                    WHITE,
                );
            }
            return;
        }
        renderer.sprite(SpriteId::Menu, vec2(0.0, 0.0));
        let custom = self.pages[self.page].start >= CAMPAIGN_COUNT;
        let stage = if custom {
            self.page - CAMPAIGN_COUNT.div_ceil(LEVELS_PER_STAGE) + 1
        } else {
            self.page + 1
        };
        let title = if custom {
            format!("Custom Stage {stage}")
        } else {
            format!("Stage {stage}")
        };
        renderer.bitmap_text(
            &title,
            vec2(if custom { 150.0 } else { 195.0 }, 75.0),
            WHITE,
        );
        for (slot, level) in self.pages[self.page].clone().enumerate() {
            let selected = self.progress.selected == level;
            let icon = if selected {
                2
            } else {
                usize::from(self.available(level))
            };
            let point = level_position(slot);
            renderer.sprite(SpriteId::LevelIcon(icon), vec2(point.x, point.y));
            let number = if custom {
                level - CAMPAIGN_COUNT + 1
            } else {
                level + 1
            };
            let x = point.x + if number < 10 { 14.0 } else { 7.0 };
            renderer.bitmap_text(
                &number.to_string(),
                vec2(x, point.y + 5.0),
                if selected { BLACK } else { WHITE },
            );
        }
        if self.pages[self.page].contains(&self.progress.selected) {
            renderer.sprite(SpriteId::Play, vec2(160.0, 240.0));
            let name = &self.levels[self.progress.selected].name;
            renderer.centered_text(name, vec2(240.0, 236.0), 16, WHITE);
        }
        renderer.sprite(SpriteId::Arrow, vec2(8.0, 102.0));
        renderer.sprite_ex(
            SpriteId::Arrow,
            vec2(409.0, 102.0),
            SpriteStyle {
                color: WHITE,
                rotation: 0.0,
                flip_x: true,
            },
        );
        if self.error.is_some() {
            renderer.bitmap_text("Level could not load", vec2(100.0, 210.0), RED);
        }
    }
}

fn level_position(slot: usize) -> Vec2 {
    if slot < 6 {
        vec2(81.0 + slot as f32 * 55.0, 120.0)
    } else {
        vec2(109.0 + (slot - 6) as f32 * 55.0, 175.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completed_game() -> Game {
        let mut game = Game::new();
        game.load("<IronJumpLevel><FPPlayer><x>0</x><y>0</y></FPPlayer><FPExit><x>0</x><y>0</y></FPExit><FPPlatform><x>0</x><y>32</y><widthSegments>3</widthSegments><heightSegments>1</heightSegments></FPPlatform></IronJumpLevel>").unwrap();
        game.update(GameInput::NONE);
        assert!(game.is_complete());
        game
    }

    #[test]
    fn viewport_survives_restart_level_changes_and_returning_from_menu() {
        let mut session = Session::new(Progress::default(), Vec::new());
        let size = vec2(712.0, 320.0);
        session.resize(size);
        session.start();
        assert_eq!(session.game.as_ref().unwrap().viewport_size(), size);
        session.start(); // Restart uses the same path as the platform's R action.
        assert_eq!(session.game.as_ref().unwrap().viewport_size(), size);
        session.next_level();
        assert_eq!(session.progress.selected, 1);
        assert_eq!(session.game.as_ref().unwrap().viewport_size(), size);
        session.menu();
        let size = vec2(320.0, 712.0);
        session.resize(size);
        session.start();
        assert_eq!(session.game.as_ref().unwrap().viewport_size(), size);
        session.resize(vec2(640.0, 320.0));
        assert_eq!(
            session.game.as_ref().unwrap().viewport_size(),
            vec2(640.0, 320.0)
        );
    }

    #[test]
    fn campaign_order_and_stage_sizes_match_iphone_game() {
        let session = Session::new(Progress::default(), Vec::new());
        assert_eq!(session.levels.len(), 35);
        assert_eq!(session.levels[0].name, "tutorial");
        assert_eq!(session.levels[29].name, "labyrinth");
        assert_eq!(session.pages, vec![0..11, 11..22, 22..30, 30..35]);
        assert!(session.levels[..30].iter().all(|level| !level.custom));
        assert!(session.levels[30..].iter().all(|level| level.custom));
    }

    #[test]
    fn locked_levels_cannot_be_selected_but_custom_levels_can() {
        let mut session = Session::new(Progress::default(), Vec::new());
        session.select(1);
        assert_eq!(session.progress.selected, 0);
        session.select(30);
        assert_eq!(session.progress.selected, 30);
        assert_eq!(session.page, 3);
        session.select(usize::MAX);
        assert_eq!(session.progress.selected, 30);
    }

    #[test]
    fn exit_advances_after_31_ticks_and_unlocks_next_campaign_level() {
        let mut session = Session::new(Progress::default(), Vec::new());
        session.game = Some(completed_game());
        session.mode = Mode::Playing;
        for _ in 0..30 {
            session.update(GameInput::NONE);
        }
        assert_eq!(session.progress.selected, 0);
        session.update(GameInput::NONE);
        assert_eq!(session.progress.selected, 1);
        assert_eq!(session.progress.unlocked, 1);
        assert_eq!(session.mode, Mode::Playing);
        assert_eq!(session.complete_ticks, 0);
    }

    #[test]
    fn last_campaign_level_shows_victory_without_entering_custom_levels() {
        let mut session = Session::new(
            Progress {
                selected: 29,
                unlocked: 29,
            },
            Vec::new(),
        );
        session.game = Some(completed_game());
        session.mode = Mode::Playing;
        for _ in 0..31 {
            session.update(GameInput::NONE);
        }
        assert_eq!(session.mode, Mode::Victory);
        assert_eq!(session.progress.selected, 29);
        for _ in 0..1000 {
            session.update(GameInput::NONE);
        }
        assert_eq!(session.win_animation, 0.7);
        session.menu();
        assert_eq!(session.mode, Mode::Menu);
        assert!(session.game.is_none());
    }

    #[test]
    fn falling_below_all_fixed_platforms_restarts_after_31_ticks() {
        let mut session = Session::new(Progress::default(), Vec::new());
        let mut game = Game::new();
        game.load("<IronJumpLevel><FPPlayer><x>0</x><y>0</y></FPPlayer><FPPlatform><x>0</x><y>-64</y><widthSegments>1</widthSegments><heightSegments>1</heightSegments></FPPlatform></IronJumpLevel>").unwrap();
        session.game = Some(game);
        session.mode = Mode::Playing;
        for _ in 0..30 {
            session.update(GameInput::NONE);
        }
        assert_eq!(session.fallen_ticks, 30);
        session.update(GameInput::NONE);
        assert_eq!(session.fallen_ticks, 0);
        assert_eq!(session.progress.selected, 0);
        assert!(!session.game.as_ref().unwrap().has_fallen());
    }

    #[test]
    fn progress_is_clamped_to_available_levels() {
        let session = Session::new(
            Progress::parse("ironjump-v1 1000 1000").unwrap(),
            Vec::new(),
        );
        assert_eq!(session.progress.unlocked, 29);
        assert!(session.progress.selected < session.levels.len());
    }

    #[test]
    fn menu_hit_boxes_select_and_start_levels_and_stop_at_page_boundaries() {
        let mut session = Session::new(
            Progress {
                selected: 0,
                unlocked: 2,
            },
            Vec::new(),
        );
        session.click(vec2(145.0, 140.0));
        assert_eq!(session.progress.selected, 1);
        session.click(vec2(200.0, 265.0));
        assert_eq!(session.mode, Mode::Playing);
        session.menu();
        session.change_page(-100);
        assert_eq!(session.page, 0);
        session.change_page(100);
        assert_eq!(session.page, 3);
    }

    #[test]
    fn menu_and_victory_draw_through_the_backend_contract() {
        use crate::drawing::test_support::{Command, RecordingRenderer};
        let mut session = Session::new(Progress::default(), Vec::new());
        let mut renderer = RecordingRenderer::default();
        session.draw(&mut renderer, "Tap: menu");
        assert!(matches!(
            renderer.commands.first(),
            Some(Command::Sprite(SpriteId::Menu, _, _))
        ));
        assert_eq!(
            renderer
                .commands
                .iter()
                .filter(|command| matches!(command, Command::Sprite(SpriteId::LevelIcon(_), _, _)))
                .count(),
            11
        );
        assert!(renderer.commands.iter().any(
            |command| matches!(command, Command::Sprite(SpriteId::Arrow, _, style) if style.flip_x)
        ));
        assert!(renderer.commands.iter().any(
            |command| matches!(command, Command::CenteredText(text, _, 16, _) if text == "tutorial")
        ));

        session.game = Some(completed_game());
        session.mode = Mode::Victory;
        renderer.commands.clear();
        session.draw(&mut renderer, "Tap: menu");
        assert!(
            renderer
                .commands
                .iter()
                .any(|command| matches!(command, Command::Rectangle(_, _)))
        );
        assert!(matches!(renderer.commands.last(),
            Some(Command::BitmapText(text, _, _)) if text == "Tap: menu"));
    }
}
