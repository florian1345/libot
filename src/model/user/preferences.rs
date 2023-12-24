use serde::Deserialize;

use crate::model::Url;

pub type Theme = String;
pub type PieceSet = String;
pub type Theme3d = String;
pub type PieceSet3d = String;
pub type SoundSet = String;
pub type Language = String;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPreferencesNoLanguage {

    // TODO some fields may be optional here?

    #[serde(default)]
    dark: bool,

    #[serde(default, rename = "transp")]
    transparent: bool,

    #[serde(rename = "bgImg")]
    background_image: Url,

    #[serde(default)]
    is_3d: bool,
    theme: Theme,
    piece_set: PieceSet,
    theme_3d: Theme3d,
    piece_set_3d: PieceSet3d,
    sound_set: SoundSet,

    // TODO resolve i32 to options

    blindfold: i32,
    auto_queen: i32,
    auto_threefold: i32,

    #[serde(rename = "takeback")]
    take_back: i32,

    #[serde(rename = "moretime")]
    more_time: i32,
    clock_tenths: i32,

    #[serde(default)]
    clock_bar: bool,

    #[serde(default)]
    clock_sound: bool,

    #[serde(default)]
    premove: bool,
    animation: i32,

    #[serde(default)]
    captured: bool,

    #[serde(default)]
    follow: bool,

    #[serde(default)]
    highlight: bool,

    #[serde(default)]
    destination: bool,
    coords: i32,
    replay: i32,
    challenge: i32,
    message: i32,
    coord_color: i32,
    submit_move: i32,
    confirm_resign: i32,
    insight_share: i32,
    keyboard_move: i32,
    zen: i32,
    move_event: i32,
    rook_castle: i32,
}

#[derive(Deserialize)]
struct UserPreferencesNested {
    prefs: UserPreferencesNoLanguage,
    language: Language
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(from = "UserPreferencesNested")]
pub struct UserPreferences {
    pub dark: bool,
    pub transparent: bool,
    pub background_image: Url,
    pub is_3d: bool,
    pub theme: Theme,
    pub piece_set: PieceSet,
    pub theme_3d: Theme3d,
    pub piece_set_3d: PieceSet3d,
    pub sound_set: SoundSet,
    pub blindfold: i32,
    pub auto_queen: i32,
    pub auto_threefold: i32,
    pub take_back: i32,
    pub more_time: i32,
    pub clock_tenths: i32,
    pub clock_bar: bool,
    pub clock_sound: bool,
    pub premove: bool,
    pub animation: i32,
    pub captured: bool,
    pub follow: bool,
    pub highlight: bool,
    pub destination: bool,
    pub coords: i32,
    pub replay: i32,
    pub challenge: i32,
    pub message: i32,
    pub coord_color: i32,
    pub submit_move: i32,
    pub confirm_resign: i32,
    pub insight_share: i32,
    pub keyboard_move: i32,
    pub zen: i32,
    pub move_event: i32,
    pub rook_castle: i32,
    pub language: Language
}

impl From<UserPreferencesNested> for UserPreferences {
    fn from(nested: UserPreferencesNested) -> UserPreferences {
        UserPreferences {
            dark: nested.prefs.dark,
            transparent: nested.prefs.transparent,
            background_image: nested.prefs.background_image,
            is_3d: nested.prefs.is_3d,
            theme: nested.prefs.theme,
            piece_set: nested.prefs.piece_set,
            theme_3d: nested.prefs.theme_3d,
            piece_set_3d: nested.prefs.piece_set_3d,
            sound_set: nested.prefs.sound_set,
            blindfold: nested.prefs.blindfold,
            auto_queen: nested.prefs.auto_queen,
            auto_threefold: nested.prefs.auto_threefold,
            take_back: nested.prefs.take_back,
            more_time: nested.prefs.more_time,
            clock_tenths: nested.prefs.clock_tenths,
            clock_bar: nested.prefs.clock_bar,
            clock_sound: nested.prefs.clock_sound,
            premove: nested.prefs.premove,
            animation: nested.prefs.animation,
            captured: nested.prefs.captured,
            follow: nested.prefs.follow,
            highlight: nested.prefs.highlight,
            destination: nested.prefs.destination,
            coords: nested.prefs.coords,
            replay: nested.prefs.replay,
            challenge: nested.prefs.challenge,
            message: nested.prefs.message,
            coord_color: nested.prefs.coord_color,
            submit_move: nested.prefs.submit_move,
            confirm_resign: nested.prefs.confirm_resign,
            insight_share: nested.prefs.insight_share,
            keyboard_move: nested.prefs.keyboard_move,
            zen: nested.prefs.zen,
            move_event: nested.prefs.move_event,
            rook_castle: nested.prefs.rook_castle,
            language: nested.language
        }
    }
}
