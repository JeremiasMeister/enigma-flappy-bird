// resources -> we load not via string but via bytes to include them in the built game
pub const BIRD: &'static [u8] = include_bytes!("res/bird.glb");
pub const PIPE: &'static [u8] = include_bytes!("res/pipe.glb");
pub const COIN: &'static [u8] = include_bytes!("res/coin.glb");
pub const BACKGROUND: &'static [u8] = include_bytes!("res/background.glb");
pub const BIRD_TEXTURE: &'static [u8] = include_bytes!("res/bird_texture.png");
pub const BACKGROUND_TEXTURE: &'static [u8] = include_bytes!("res/background_texture.png");
pub const HEART_TEXTURE: &'static [u8] = include_bytes!("res/heart.png");
pub const FONT_PRESS_START: &'static [u8] = include_bytes!("res/PrStart.ttf");

pub const BACKGROUND_MUSIC: &'static [u8] = include_bytes!("res/background-music.ogg");
pub const HIT_SOUND: &'static [u8] = include_bytes!("res/hit-sound.ogg");
pub const COLLECT_SOUND: &'static [u8] = include_bytes!("res/collect-sound.ogg");
pub const COLLECT_SOUND_TEN: &'static [u8] = include_bytes!("res/collect-sound-2.ogg");
pub const WUSH_SOUND: &'static [u8] = include_bytes!("res/wush.ogg");
pub const GAME_OVER_SOUND: &'static [u8] = include_bytes!("res/game-over.ogg");
pub const HIGHSCORE_FILE: &str = "enigma-3d_flappy_bird_highscore.txt";