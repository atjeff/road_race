use rand::prelude::*;
use rusty_engine::prelude::*;

const PLAYER_SPEED: f32 = 250.0;
const ROAD_SPEED: f32 = 400.0;

#[derive(Resource)]
struct GameState {
    health: u8,
    lost: bool,
}

fn main() {
    let mut game = Game::new();

    // game setup goes here
    let state = GameState {
        health: 5,
        lost: false,
    };

    game.audio_manager
        .play_music("music/Whimsical Popsicle.ogg", 0.2);

    // Add health display
    let health_display = game.add_text("health_display", "Health: 5");
    health_display.translation.x = -575.0;
    health_display.translation.y = 325.0;

    // Spawn player car
    let player = game.add_sprite("player", SpritePreset::RacingCarBlue);
    player.translation.x = -500.0;
    player.layer = 10.0;
    player.collision = true;

    // Spawn 10 road lines
    for i in 0..10 {
        let label = format!("road_line_{}", i);
        let road_line = game.add_sprite(label, SpritePreset::RacingBarrierWhite);
        road_line.scale = 0.1;
        road_line.translation.x = -600.0 + 150.0 * i as f32;
    }

    // Spawn obstacles
    let obstacle_presets = vec![
        SpritePreset::RacingBarrelBlue,
        SpritePreset::RacingBarrelRed,
        SpritePreset::RacingConeStraight,
    ];

    for (i, preset) in obstacle_presets.into_iter().enumerate() {
        let obstacle = game.add_sprite(format!("obstacle_{}", i), preset);
        obstacle.layer = 5.0;
        obstacle.collision = true;
        obstacle.translation.x = thread_rng().gen_range(800.0..1600.0);
        obstacle.translation.y = thread_rng().gen_range(-300.0..300.0);
    }

    game.add_logic(game_logic);
    game.run(state);
}

// This code runs every frame
fn game_logic(engine: &mut Engine, state: &mut GameState) {
    // If the game is lost, return fast
    if state.lost {
        return;
    }

    // Handle keyboard input to let the user steer up or down
    let mut direction = 0.0;

    if engine
        .keyboard_state
        .pressed_any(&[KeyCode::W, KeyCode::Up])
    {
        direction += 1.0;
    }

    if engine
        .keyboard_state
        .pressed_any(&[KeyCode::S, KeyCode::Down])
    {
        direction -= 1.0;
    }

    // Move the player car
    let player = engine.sprites.get_mut("player").unwrap();
    player.translation.y += direction * PLAYER_SPEED * engine.delta_f32;
    player.rotation = direction * 0.15;

    // If they go out of bounds they die
    if player.translation.y < -360.0 || player.translation.y > 360.0 {
        state.health = 0;

        engine.audio_manager.play_sfx("sfx/forcefield1.ogg", 0.5);
    }

    // Move the road lines
    for sprite in engine.sprites.values_mut() {
        if sprite.label.contains("road_line") {
            sprite.translation.x -= ROAD_SPEED * engine.delta_f32;

            // If the road line goes off the screen, reset it
            if sprite.translation.x < -675.0 {
                sprite.translation.x += 1500.0;
            }
        }
    }

    // Move the obstacles
    for sprite in engine.sprites.values_mut() {
        if sprite.label.contains("obstacle") {
            sprite.translation.x -= ROAD_SPEED * engine.delta_f32;

            // If the sprite goes off the screen, reset it
            if sprite.translation.x < -800.0 {
                sprite.translation.x = thread_rng().gen_range(800.0..1600.0);
                sprite.translation.y = thread_rng().gen_range(-300.0..300.0);
            }
        }
    }

    // Check for collisions between player and obstacles
    for event in engine.collision_events.drain(..) {
        println!("Collision between {} and {}", event.pair.0, event.pair.1);

        if event.state == CollisionState::Begin && event.pair.one_starts_with("player") {
            if state.health > 0 {
                state.health -= 1;
            }

            // Find obstacle in pair
            if let Some(obstacle_label) = event
                .pair
                .array()
                .iter()
                .find(|obstacle| obstacle.starts_with("obstacle"))
            {
                // Reset the obstacle
                let obstacle = engine.sprites.get_mut(*obstacle_label).unwrap();

                obstacle.translation.x = thread_rng().gen_range(800.0..1600.0);
                obstacle.translation.y = thread_rng().gen_range(-300.0..300.0);
            };

            engine.audio_manager.play_sfx("sfx/impact2.ogg", 0.5);
        }
    }

    // Update health display
    let health_display = engine.texts.get_mut("health_display").unwrap();
    health_display.value = format!("Health: {}", state.health);

    if state.health == 0 {
        state.lost = true;

        let game_over = engine.add_text("game over", "Game Over");
        game_over.font_size = 128.0;

        engine.audio_manager.stop_music();
        engine.audio_manager.play_sfx(SfxPreset::Jingle3, 0.5);
    }
}
