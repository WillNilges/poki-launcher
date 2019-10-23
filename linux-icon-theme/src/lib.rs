// Implementation based on freedesktop.org spec https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html

use failure::{Error, Fail};
use ini::Ini;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

const SEARCH_PATHS: &[&str] = &[
    "~/.icons",
    "$XDG_DATA_DIRS/icons",
    "~/.local/share/icons",
    "/usr/share/pixmaps",
    "/usr/share/icons",
];

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "{} is missing prop {}", path, name)]
    MissingProp { path: String, name: &'static str },
    #[fail(display = "{} is missing the section [Icon Theme]", path)]
    MissingIndexTheme { path: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct IconCache {
    pub themes: Vec<IconTheme>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut themes = Vec::new();
        for search_path in SEARCH_PATHS {
            let expanded = shellexpand::full(&search_path).unwrap().into_owned();
            for entry in read_dir(&expanded).unwrap() {
                let entry = entry.unwrap();
            }
        }
        IconCache { themes }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IconTheme {
    pub name: String,
    pub comment: String,
    inherits: Vec<String>,
    directories: Vec<Directory>,
    scaled_directories: Vec<Directory>,
    pub hidden: bool,
    pub example: Option<String>,
}

impl IconTheme {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let mut index_path = path.as_ref().to_path_buf();
        index_path.push("index.theme");
        let index_file = Ini::load_from_file(index_path.as_path())?;
        let theme_section = index_file.section(Some("Icon Theme".to_owned())).ok_or(
            ParseError::MissingIndexTheme {
                path: path_str.clone(),
            },
        )?;
        let name = theme_section
            .get("Name")
            .ok_or(ParseError::MissingProp {
                path: path_str.clone(),
                name: "Name",
            })?
            .clone();
        let comment = theme_section
            .get("Comment")
            .ok_or(ParseError::MissingProp {
                path: path_str.clone(),
                name: "Comment",
            })?
            .clone();
        let inherits = match theme_section.get("Inherits") {
            Some(value) => value.split(",").map(|s| s.to_owned()).collect(),
            None => Vec::new(),
        };
        let (directories, errors): (Vec<_>, Vec<_>) = theme_section
            .get("Directories")
            .ok_or(ParseError::MissingProp {
                path: path_str.clone(),
                name: "Directories",
            })?
            .split(",")
            .map(|name| Directory::from(&name, &index_file))
            .partition(Result::is_ok);
        let directories = directories.into_iter().map(Result::unwrap).collect();
        // let mut directories = Vec::new();
        // let mut theme_info = None;
        // for entry in path.as_ref().read_dir()? {
        //     let entry = entry?;
        //     let path = entry.path();
        //     if path.is_file() && path.file_name().unwrap().to_string_lossy() == "index.theme" {
        //     } else if path.is_dir() {
        //         directories.push(path);
        //     }
        // }
        Ok(IconTheme {
            name,
            comment,
            inherits,
            directories,
            scaled_directories: Vec::new(),
            hidden: false,
            example: None,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Directory {
    path: PathBuf,
    size: u32,
    // Spec says this is an int (should investigate if some things use fractional scaling)
    scale: u32,
    context: Option<String>,
    type_: DirectoryType,
    max_size: u32,
    min_size: u32,
    threshold: u32,
}

impl Directory {
    pub fn from(name: &str, file: &ini::ini::Ini) -> Result<Self, Error> {
        unimplemented!();
    }
}

impl Default for Directory {
    fn default() -> Self {
        Directory {
            path: Default::default(),
            size: Default::default(),
            scale: 1,
            context: None,
            type_: DirectoryType::Threshold,
            max_size: Default::default(),
            min_size: Default::default(),
            threshold: 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DirectoryType {
    Fixed,
    Scalable,
    Threshold,
}
