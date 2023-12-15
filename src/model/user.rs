use serde::Deserialize;

pub type UserId = String;
pub type Rating = i32;
pub type AiLevel = i32;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Title {
    Gm,
    Wgm,
    Im,
    Wim,
    Fm,
    Wfm,
    Nm,
    Wnm,
    Cm,
    Wcm,
    Lm,
    Bot
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct User {
    // TODO really so many options?
    pub rating: Option<Rating>,
    pub provisional: Option<bool>,
    pub online: Option<bool>,
    pub id: UserId,
    pub name: String,
    pub title: Option<Title>,
    pub patron: Option<bool>
}
