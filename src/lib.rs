use bevy::{
    prelude::*,
    sprite::collide_aabb::*,
};

#[derive(Copy, Clone)]
pub struct GameOptions {
    pub size: Vec2,
    /// Center position of the game, players and ball are placed relative to this
    /// position and with a z-Coordinate which is 1 higher.
    pub position: Vec3,
    /// The background color for the entire game.
    pub background: Color,
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            size: Vec2::new(600., 400.),
            position: Vec3::default(),
            background: Color::BLACK,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PlayerOptions {
    /// The colors for the players (colors.0 is for player 1; colors.1 is for player 2).
    pub colors: (Color, Color),
    pub size: Vec2,
    /// Up and down keys to control player one (the left).
    pub player1_keys: (KeyCode, KeyCode),
    /// Up and down keys to control player two (the right).
    pub player2_keys: (KeyCode, KeyCode),
    pub speed: f32,
}

impl Default for PlayerOptions {
    fn default() -> Self {
        Self {
            colors: (Color::WHITE, Color::WHITE),
            size: Vec2::new(5., 50.),
            player1_keys: (KeyCode::W, KeyCode::S),
            player2_keys: (KeyCode::Up, KeyCode::Down),
            speed: 200.,
        }
    }
}

#[derive(Copy, Clone)]
pub struct BallOptions {
    pub color: Color,
    pub size: Vec2,
    /// Function which gets used to get the velocity with which the ball should start.
    pub start_velocity: fn() -> Vec2,
    /// The factor by which the velocity gets multiplied periodically.
    pub speedup_factor: f32,
    /// The period (in seconds) the balls velocity gets incremented.
    pub speedup_time: f32,
}

impl Default for BallOptions {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            size: Vec2::new(15., 15.),
            start_velocity: || Vec2::new(30., 15.),
            speedup_factor: 1.1,
            speedup_time: 1.5,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ScoreDisplayOptions {
    font_path: &'static str,
    font_size: f32,
    font_color: Color,
}

impl Default for ScoreDisplayOptions {
    fn default() -> Self {
        Self {
            font_path: "fonts/FiraMono-Medium.ttf",
            font_size: 20.,
            font_color: Color::WHITE,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PongOptions {
    pub game: GameOptions,
    pub player: PlayerOptions,
    pub ball: BallOptions,
    /// Determines whether the default player score display should be used and how the score gets displayed.
    pub score_display_options: Option<ScoreDisplayOptions>,
}

impl Default for PongOptions {
    fn default() -> Self {
        Self {
            game: Default::default(),
            player: Default::default(),
            ball: Default::default(),
            score_display_options: Some(Default::default()),
        }
    }
}

impl PongOptions {
    pub fn color_for(&self, player: &Player) -> Color {
        match player {
            Player::Player1 => self.player.colors.0,
            Player::Player2 => self.player.colors.1,
        }
    }
    pub fn up_for(&self, player: &Player) -> KeyCode {
        match player {
            Player::Player1 => self.player.player1_keys.0,
            Player::Player2 => self.player.player2_keys.0,
        }
    }
    pub fn down_for(&self, player: &Player) -> KeyCode {
        match player {
            Player::Player1 => self.player.player1_keys.1,
            Player::Player2 => self.player.player2_keys.1,
        }
    }
}

pub struct PongPlugin;

impl Plugin for PongPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScoredPointEvent>()
            .add_startup_system(setup_pong)
            .add_system(handle_player_input.label("a"))
            .add_system(speedup_ball.label("a"))
            .add_system(apply_ball_velocity.label("b").after("a"))
            .add_system(check_point_scored.label("b").after("a"))
            .add_system(update_score_text.label("c").after("b"));
    }
}

#[derive(Component)]
pub struct PongGame;

#[derive(Component)]
pub struct Ball;

impl Ball {
    fn start_position(options: &PongOptions) -> Vec3 {
        Vec3::new(0., 0., options.game.position.z + 1.)
    }
}

#[derive(Component)]
pub struct Velocity(Vec2);

struct BallSpeedupTimer(Timer);

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    fn start_position(&self, options: &PongOptions) -> Vec3 {
        let x = options.game.size.x / 2. - options.player.size.x;
        let z = options.game.position.z + 1.;
        match self {
            Player::Player1 => Vec3::new(-x, 0., z),
            Player::Player2 => Vec3::new(x, 0., z),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Score(u16);

#[derive(Component)]
pub struct ScoreDisplayText;

pub struct ScoredPointEvent(Player, Score);

pub type IsBall = (With<Ball>, Without<Player>);
pub type IsPlayer = (With<Player>, Without<Ball>);

fn setup_pong(mut commands: Commands, asset_server: Res<AssetServer>, pong_options: Option<Res<PongOptions>>) {
    let options = match pong_options {
        Some(opt) => *opt,
        None => {
            commands.insert_resource(PongOptions::default());
            PongOptions::default()
        }
    };

    let entity = commands.spawn()
        .insert(PongGame)
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: options.game.background,
                custom_size: Some(options.game.size),
                ..Default::default()
            },
            transform: Transform::from_translation(options.game.position),
            ..Default::default()
        })
        .with_children(|parent| {
            for player in [Player::Player1, Player::Player2].iter() {
                parent.spawn()
                    .insert(*player)
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: options.color_for(player),
                            custom_size: Some(options.player.size),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(player.start_position(&options)),
                        ..Default::default()
                    })
                    .insert(Score(0))
                    .insert(Velocity(Vec2::default()));
            }
            parent.spawn().insert(Ball)
                .insert_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: options.ball.color,
                        custom_size: Some(options.ball.size),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Ball::start_position(&options)),
                    ..Default::default()
                })
                .insert(Velocity((options.ball.start_velocity)()));
        }).id();
    
    if options.score_display_options.is_some() {
        let score_options = options.score_display_options.unwrap();
        let text_style = TextStyle {
                        font: asset_server.load(score_options.font_path),
                        font_size: score_options.font_size,
                        color: score_options.font_color,
        };
        let section = |s: &str| TextSection { value: s.into(), style: text_style.clone() };

        commands.entity(entity).with_children(|parent| {
            parent.spawn().insert(ScoreDisplayText)
                .insert_bundle(Text2dBundle {
                    text: Text {
                        sections: vec![ section("0"), section(":"), section("0") ],
                        alignment: TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        },
                    },
                    transform: Transform::from_translation(Vec3::new(
                        0.,
                        options.game.size.y / 2. - score_options.font_size * (2. / 3.),
                        options.game.position.z + 1.
                    )),
                    ..Default::default()
                });
        });
    }

    commands.insert_resource(BallSpeedupTimer(
            Timer::from_seconds(options.ball.speedup_time, true)
    ));
}

fn handle_player_input(
    options: Res<PongOptions>,
    time: Res<Time>,
    key_input: Res<Input<KeyCode>>,
    mut players: Query<(&Player, &mut Transform)>
) {
    let delta = time.delta_seconds();
    let movement = options.player.speed * delta;
    let hps = options.player.size.y / 2.;
    let hgs = options.game.size.y / 2.;

    for (player, mut transform) in players.iter_mut() {
        let y = &mut transform.translation.y;
        if key_input.pressed(options.up_for(player)) && (*y + hps + movement) <= hgs {
            *y += movement;
        }
        if key_input.pressed(options.down_for(player)) && (*y - hps - movement) >= -hgs {
            *y -= movement;
        }
    }
}

fn speedup_ball(
    mut ball_timer: ResMut<BallSpeedupTimer>,
    time: Res<Time>,
    options: Res<PongOptions>,
    mut ball_velocities: Query<&mut Velocity, IsBall>,
) {
    if !ball_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for mut vel in ball_velocities.iter_mut() {
        vel.0 *= options.ball.speedup_factor;
    }
}

fn apply_ball_velocity(
    time: Res<Time>,
    options: Res<PongOptions>,
    mut balls: Query<(&mut Transform, &mut Velocity), IsBall>,
    players: Query<&Transform, IsPlayer>,
) {
    let delta = time.delta_seconds();

    let hgs = options.game.size.y / 2.;
    let hbs = options.ball.size.y / 2.;
    for (mut trans, mut vel) in balls.iter_mut() {
        trans.translation.x += vel.0.x * delta;
        trans.translation.y += vel.0.y * delta;

        for p_trans in players.iter() {
            if let Some(col) = collide(
                p_trans.translation, options.player.size,
                trans.translation, options.ball.size
            ) {
                match col {
                    Collision::Left | Collision::Right => vel.0.x *= -1.,
                    Collision::Top | Collision::Bottom => vel.0.y *= -1.,
                }
            }
        }

        if trans.translation.y + hbs >= hgs {           // Ball hits top
            vel.0.y *= -1.;
            trans.translation.y = hgs - hbs;
        } else if trans.translation.y - hbs <= -hgs {   // Ball hits bottom
            vel.0.y *= -1.;
            trans.translation.y = -hgs + hbs;
        }
    }
}

fn check_point_scored(
    options: Res<PongOptions>,
    mut event_writer: EventWriter<ScoredPointEvent>,
    mut balls: Query<(&mut Transform, &mut Velocity), IsBall>,
    mut players: Query<(&Player, &mut Transform, &mut Score), IsPlayer>
) {
    let max_x = options.game.size.x / 2.;
    let min_x = -max_x;
    let hbsx = options.ball.size.x / 2.;

    let reset_ball = |mut t: &mut Transform, mut v: &mut Velocity| {
        t.translation = Vec3::new(0., 0., 1.);
        v.0 = (options.ball.start_velocity)();
    };
    let mut reset_player_and_send_event = |scoring_player: Player| {
        for (player, mut p_trans, mut score) in players.iter_mut() {
            if *player == scoring_player {
                score.0 += 1;
                event_writer.send(ScoredPointEvent(*player, *score));
            }
            p_trans.translation.y = 0.;
        }
    };

    for (mut b_trans, mut vel) in balls.iter_mut() {
        if b_trans.translation.x - hbsx <= min_x {
            reset_ball(&mut b_trans, &mut vel);
            reset_player_and_send_event(Player::Player2);
        } else if b_trans.translation.x + hbsx >= max_x {
            reset_ball(&mut b_trans, &mut vel);
            reset_player_and_send_event(Player::Player1);
        }
    }
}

fn update_score_text(
    options: Res<PongOptions>,
    mut event_reader: EventReader<ScoredPointEvent>,
    mut score_text: Query<&mut Text, With<ScoreDisplayText>>,
) {
    if options.score_display_options.is_none() {
        return;
    }

    for ScoredPointEvent(player, Score(points)) in event_reader.iter() {
        for mut text in score_text.iter_mut() {
            match player {
                Player::Player1 => text.sections[0].value = format!("{}", points),
                Player::Player2 => text.sections[2].value = format!("{}", points),
            }
        }
    }
}