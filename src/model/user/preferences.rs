use std::ops::{BitOr, BitOrAssign};
use serde::{Deserialize, Deserializer};
use serde::de::Error as DeserializeError;
use serde_repr::Deserialize_repr;

use crate::model::Url;

pub type Theme = String;
pub type PieceSet = String;
pub type Theme3d = String;
pub type PieceSet3d = String;
pub type SoundSet = String;
pub type Language = String;

/// Preference for piece animation.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum PieceAnimation {
    None = 0,
    Fast = 1,
    Normal = 2,
    Slow = 3
}

/// Preference for automatically promoting to Queen.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum AutoQueen {
    Never = 1,
    WhenPreMoving = 2,
    Always = 3
}

/// Preference for automatically claiming draw on threefold repetition.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum AutoThreefold {
    Never = 1,
    WhenLessThan30Seconds = 2,
    Always = 3
}

/// Preference for which players to let challenge you.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ChallengeFilter {
    Never = 1,
    IfRatingWithin300 = 2,
    OnlyFriends = 3,
    IfRegistered = 4,
    Always = 5
}

/// Preference for displaying tenths of seconds on the clock.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ClockTenths {
    Never = 0,
    WhenLessThan10Seconds = 1,
    Always = 2
}

/// Preference for displaying board coordinates (A-H, 1-8).
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Coordinates {

    /// Do not display coordinates.
    None = 0,

    /// Display coordinates inside the board.
    Inside = 1,

    /// Display coordinates outside the board.
    Outside = 2
}

/// Preference for sharing your Chess insights data.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum InsightShare {
    WithNobody = 0,
    WithFriends = 1,
    WithEverybody = 2
}

/// Preference for which players to let message you.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum MessageFilter {
    OnlyExistingConversations = 1,
    OnlyFriends = 2,
    Always = 3
}

/// Preference for giving more time.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum MoreTime {
    Never = 1,
    CasualOnly = 2,
    Always = 3
}

/// Preference for how you move pieces.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum MoveEvent {
    ClickTwoSquares = 0,
    DragPiece = 1,
    Either = 2
}

/// Preference for displaying a move list while playing.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Replay {
    Never = 0,
    SlowGames = 1,
    Always = 2
}

/// Preference for castling method.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum CastlingMethod {
    KingTwoSquares = 0,
    KingOntoRook = 1
}

/// Preference for take backs with opponent approval.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum TakeBack {
    Never = 1,
    CasualOnly = 2,
    Always = 3
}

/// Preference for activating Zen mode.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ZenMode {
    No = 0,
    Yes = 1,
    InGameOnly = 2
}

/// Preferences on whether moves have to be confirmed depending on the time control.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct MoveConfirmations(u8);

impl MoveConfirmations {
    pub const EMPTY: MoveConfirmations = MoveConfirmations(0);
    pub const UNLIMITED: MoveConfirmations = MoveConfirmations(1);
    pub const CORRESPONDENCE: MoveConfirmations = MoveConfirmations(2);
    pub const CLASSICAL: MoveConfirmations = MoveConfirmations(4);
    pub const RAPID: MoveConfirmations = MoveConfirmations(8);
    pub const BLITZ: MoveConfirmations = MoveConfirmations(16);

    fn contains(self, mask: MoveConfirmations) -> bool {
        self.0 & mask.0 != 0
    }

    /// Indicates whether move confirmation is required in Unlimited time control.
    pub fn for_unlimited(self) -> bool {
        self.contains(Self::UNLIMITED)
    }

    /// Indicates whether move confirmation is required in Correspondence time control.
    pub fn for_correspondence(self) -> bool {
        self.contains(Self::CORRESPONDENCE)
    }

    /// Indicates whether move confirmation is required in Classical time control.
    pub fn for_classical(self) -> bool {
        self.contains(Self::CLASSICAL)
    }

    /// Indicates whether move confirmation is required in Rapid time control.
    pub fn for_rapid(self) -> bool {
        self.contains(Self::RAPID)
    }

    /// Indicates whether move confirmation is required in Blitz time control.
    pub fn for_blitz(self) -> bool {
        self.contains(Self::BLITZ)
    }
}

impl BitOr for MoveConfirmations {
    type Output = MoveConfirmations;

    fn bitor(self, rhs: MoveConfirmations) -> MoveConfirmations {
        MoveConfirmations(self.0 | rhs.0)
    }
}

impl BitOrAssign for MoveConfirmations {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPreferencesNoLanguage {

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

    #[serde(deserialize_with = "deserialize_bool_from_integer")]
    blindfold: bool,
    auto_queen: AutoQueen,
    auto_threefold: AutoThreefold,

    #[serde(rename = "takeback")]
    take_back: TakeBack,

    #[serde(rename = "moretime")]
    more_time: MoreTime,
    clock_tenths: ClockTenths,

    #[serde(default)]
    clock_bar: bool,

    #[serde(default)]
    clock_sound: bool,

    #[serde(default)]
    premove: bool,
    animation: PieceAnimation,

    #[serde(default)]
    captured: bool,

    #[serde(default)]
    follow: bool,

    #[serde(default)]
    highlight: bool,

    #[serde(default)]
    destination: bool,
    coords: Coordinates,
    replay: Replay,
    challenge: ChallengeFilter,
    message: MessageFilter,
    submit_move: MoveConfirmations,

    #[serde(deserialize_with = "deserialize_bool_from_integer")]
    confirm_resign: bool,
    insight_share: InsightShare,

    #[serde(deserialize_with = "deserialize_bool_from_integer")]
    keyboard_move: bool,
    zen: ZenMode,

    #[serde(deserialize_with = "deserialize_bool_from_integer")]
    ratings: bool,
    move_event: MoveEvent,

    #[serde(rename = "rookCastle")]
    castling_method: CastlingMethod
}

fn deserialize_bool_from_integer<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>
{
    let integer = i64::deserialize(deserializer)?;

    match integer {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DeserializeError::custom(format!("invalid integer for bool: {}", integer)))
    }
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
    pub blindfold: bool,
    pub auto_queen: AutoQueen,
    pub auto_threefold: AutoThreefold,
    pub take_back: TakeBack,
    pub more_time: MoreTime,
    pub clock_tenths: ClockTenths,
    pub clock_bar: bool,
    pub clock_sound: bool,
    pub premove: bool,
    pub animation: PieceAnimation,
    pub captured: bool,
    pub follow: bool,
    pub highlight: bool,
    pub destination: bool,
    pub coords: Coordinates,
    pub replay: Replay,
    pub challenge: ChallengeFilter,
    pub message: MessageFilter,
    pub submit_move: MoveConfirmations,
    pub confirm_resign: bool,
    pub insight_share: InsightShare,
    pub keyboard_move: bool,
    pub zen: ZenMode,
    pub ratings: bool,
    pub move_event: MoveEvent,
    pub castling_method: CastlingMethod,
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
            submit_move: nested.prefs.submit_move,
            confirm_resign: nested.prefs.confirm_resign,
            insight_share: nested.prefs.insight_share,
            keyboard_move: nested.prefs.keyboard_move,
            zen: nested.prefs.zen,
            ratings: nested.prefs.ratings,
            move_event: nested.prefs.move_event,
            castling_method: nested.prefs.castling_method,
            language: nested.language
        }
    }
}
