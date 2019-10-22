// Implementation based on freedesktop.org spec https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html

use std::path::PathBuf;

const SEARCH_PATHS: &[&str] = &[
    "~/.icons",
    "$XDG_DATA_DIRS/icons",
    "~/.local/share/icons",
    "/usr/share/pixmaps",
];

#[derive(Debug, PartialEq, Eq)]
pub struct IconCache;

impl IconCache {
    pub fn new() {}
}

#[derive(Debug, PartialEq, Eq)]
pub struct IconTheme {
    pub name: String,
    pub comment: String,
    inherits: Vec<String>,
    directories: Vec<PathBuf>,
    scaled_directories: Vec<PathBuf>,
    pub hidden: bool,
    pub example: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Icon {
    path: PathBuf,
    size: u32,
    // Spec says this is an int (should investigate if some things use fractional scaling)
    scale: u32,
    context: Option<String>,
    type_: IconType,
    max_size: u32,
    min_size: u32,
    threshold: u32,
}

impl Default for Icon {
    fn default() -> Self {
        Icon {
            path: Default::default(),
            size: Default::default(),
            scale: 1,
            context: None,
            type_: IconType::Threshold,
            max_size: Default::default(),
            min_size: Default::default(),
            threshold: 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum IconType {
    Fixed,
    Scalable,
    Threshold,
}
