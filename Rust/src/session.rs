use crate::{
    assets::Assets,
    game::Game,
    levels::{self, CAMPAIGN_COUNT, LEVELS_PER_STAGE, Level},
};
use macroquad::prelude::*;
use std::{ops::Range, path::Path};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub selected: usize,
    pub unlocked: usize,
}

impl Progress {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(error.to_string()),
        };
        let mut fields = text.split_whitespace();
        if fields.next() != Some("ironjump-v1") {
            return Err("Unrecognized progress file".into());
        }
        let selected = fields
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or("Invalid selected level")?;
        let unlocked = fields
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or("Invalid unlocked level")?;
        if fields.next().is_some() {
            return Err("Unexpected progress data".into());
        }
        Ok(Self {
            selected,
            unlocked: unlocked.min(CAMPAIGN_COUNT - 1),
        })
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let temporary = path.with_extension("tmp");
        std::fs::write(
            &temporary,
            format!("ironjump-v1 {} {}\n", self.selected, self.unlocked),
        )?;
        std::fs::rename(temporary, path)
    }
}

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

    pub fn start(&mut self) {
        let mut game = Game::new();
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

    pub fn update(&mut self, input: Vec2) {
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

    pub fn draw(&self, assets: &Assets) {
        if let Some(game) = &self.game {
            game.draw(assets, self.levels[self.progress.selected].custom);
            if self.mode == Mode::Victory {
                draw_rectangle(
                    0.0,
                    0.0,
                    480.0,
                    320.0,
                    Color::new(0.0, 0.0, 0.0, self.win_animation),
                );
                let y = 320.0 - self.win_animation * 280.0;
                assets.win.draw(120.0, y);
                for x in 0..8 {
                    assets.elevator[2].draw(120.0 + x as f32 * 32.0, y + 64.0);
                }
                assets.text("Enter: menu", 160.0, 275.0, WHITE);
            }
            return;
        }
        assets.menu.draw(0.0, 0.0);
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
        assets.text(&title, if custom { 150.0 } else { 195.0 }, 75.0, WHITE);
        for (slot, level) in self.pages[self.page].clone().enumerate() {
            let selected = self.progress.selected == level;
            let icon = if selected {
                2
            } else {
                usize::from(self.available(level))
            };
            let point = level_position(slot);
            assets.level_icons[icon].draw(point.x, point.y);
            let number = if custom {
                level - CAMPAIGN_COUNT + 1
            } else {
                level + 1
            };
            let x = point.x + if number < 10 { 14.0 } else { 7.0 };
            assets.text(
                &number.to_string(),
                x,
                point.y + 5.0,
                if selected { BLACK } else { WHITE },
            );
        }
        if self.pages[self.page].contains(&self.progress.selected) {
            assets.play.draw(160.0, 240.0);
            let name = &self.levels[self.progress.selected].name;
            draw_text(
                name,
                240.0 - measure_text(name, None, 16, 1.0).width * 0.5,
                236.0,
                16.0,
                WHITE,
            );
        }
        assets.arrow.draw(8.0, 102.0);
        assets.arrow.draw_ex(vec2(409.0, 102.0), WHITE, 0.0, true);
        if self.error.is_some() {
            assets.text("Level could not load", 100.0, 210.0, RED);
        }
    }

    /// Draw after the game render target is presented, so small text is
    /// rasterized at window resolution instead of enlarged from 480 × 320.
    pub fn draw_controls(&self, view: Rect) {
        if self.mode != Mode::Menu {
            return;
        }
        let scale = view.w / 480.0;
        let height = 18.0 * scale;
        let top = view.bottom() - height;
        draw_rectangle(view.x, top, view.w, height, Color::new(0.0, 0.0, 0.0, 0.85));

        let text = "Arrows / A,D: move   Space: jump   R: restart   Esc: menu";
        let font_size = (14.0 * scale).round().max(1.0) as u16;
        let measured = measure_text(text, None, font_size, 1.0);
        let font_scale = ((view.w - 24.0 * scale) / measured.width).min(1.0);
        let dimensions = measure_text(text, None, font_size, font_scale);
        draw_text_ex(
            text,
            view.x + (view.w - dimensions.width) * 0.5,
            top + (height - dimensions.height) * 0.5 + dimensions.offset_y,
            TextParams {
                font_size,
                font_scale,
                color: WHITE,
                ..Default::default()
            },
        );
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
        game.update(Vec2::ZERO);
        assert!(game.is_complete());
        game
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
            session.update(Vec2::ZERO);
        }
        assert_eq!(session.progress.selected, 0);
        session.update(Vec2::ZERO);
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
            session.update(Vec2::ZERO);
        }
        assert_eq!(session.mode, Mode::Victory);
        assert_eq!(session.progress.selected, 29);
        for _ in 0..1000 {
            session.update(Vec2::ZERO);
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
            session.update(Vec2::ZERO);
        }
        assert_eq!(session.fallen_ticks, 30);
        session.update(Vec2::ZERO);
        assert_eq!(session.fallen_ticks, 0);
        assert_eq!(session.progress.selected, 0);
        assert!(!session.game.as_ref().unwrap().has_fallen());
    }

    #[test]
    fn progress_roundtrips_and_clamps_unlocked_levels() {
        let path =
            std::env::temp_dir().join(format!("ironjump-progress-test-{}.txt", std::process::id()));
        let progress = Progress {
            selected: 12,
            unlocked: 15,
        };
        progress.save(&path).unwrap();
        assert_eq!(Progress::load(&path).unwrap(), progress);
        std::fs::write(&path, "ironjump-v1 1000 1000").unwrap();
        let session = Session::new(Progress::load(&path).unwrap(), Vec::new());
        assert_eq!(session.progress.unlocked, 29);
        assert!(session.progress.selected < session.levels.len());
        std::fs::write(&path, "broken").unwrap();
        assert!(Progress::load(&path).is_err());
        std::fs::remove_file(&path).unwrap();
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
}
