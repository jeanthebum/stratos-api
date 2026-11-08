// Runs while menus are open as well as while playing.
fn stratos_lifecycle(game: &mut crate::game::Game<'_>) {
    game.stratos_sync_enabled();
}
