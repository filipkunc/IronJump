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
