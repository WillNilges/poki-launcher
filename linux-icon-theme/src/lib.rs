// Implementation based on freedesktop.org spec https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html

use failure::{Error, Fail};
use ini::Ini;
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const SEARCH_PATHS: &[&str] = &[
    "~/.icons",
    "$XDG_DATA_DIRS/icons",
    "~/.local/share/icons",
    "/usr/share/pixmaps",
    "/usr/share/icons",
];
const DEFAULT_THEME: &str = "hicolor";

#[derive(Debug, Fail)]
pub enum CreateError {
    #[fail(display = "{} is missing prop {}", path, name)]
    MissingProp { path: String, name: &'static str },
    #[fail(display = "{} is missing the section [Icon Theme]", path)]
    MissingIndexTheme { path: String },
    #[fail(
        display = "In entry {} property {} has an invalid value {}",
        path, name, value
    )]
    InvalidPropVal {
        path: String,
        name: &'static str,
        value: String,
    },
    #[fail(display = "{} missing section for {} directory", path, name)]
    MissingDirSection { path: String, name: String },
    #[fail(
        display = "{} is missing prop {} for directory {}",
        path, prop_name, name
    )]
    MissingDirProp {
        path: String,
        name: String,
        prop_name: &'static str,
    },
    #[fail(display = "Missing ext")]
    MissingExt,
}

#[derive(Debug)]
pub struct IconCache {
    themes: HashMap<String, IconTheme>,
}

impl IconCache {
    pub fn create() -> Result<(Self, Vec<Error>), Error> {
        let mut themes = HashMap::new();
        let mut errors = Vec::new();
        for search_path in SEARCH_PATHS {
            let expanded = shellexpand::full(&search_path)?.into_owned();
            match read_dir(&expanded) {
                Ok(dir) => {
                    for entry in dir {
                        let entry = entry?;
                        match IconTheme::from_path(entry.path()) {
                            Ok((icon_theme, errs)) => {
                                themes.insert(icon_theme.name.clone(), icon_theme);
                                errors.extend(errs);
                            }
                            Err(e) => errors.push(e),
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        Ok((IconCache { themes }, errors))
    }

    pub fn get_icon(&self, theme_name: &str, icon_name: &str, size: u32) -> Option<PathBuf> {
        let theme = self.themes.get(theme_name)?;
        for directory in &theme.directories {
            for (name, ext) in &directory.icons {
                if name == icon_name {
                    let mut path = directory.path.clone();
                    path.push(format!("{}.{}", name, ext));
                    return Some(path);
                }
            }
        }
        if theme_name == DEFAULT_THEME {
            None
        } else {
            for parent in &theme.inherits {
                if let Some(path) = self.get_icon(parent, icon_name, size) {
                    return Some(path);
                }
            }
            self.get_icon(DEFAULT_THEME, icon_name, size)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IconTheme {
    pub name: String,
    pub display_name: String,
    pub comment: String,
    inherits: Vec<String>,
    directories: Vec<Directory>,
    scaled_directories: Vec<Directory>,
    pub hidden: bool,
    pub example: Option<String>,
}

impl IconTheme {
    pub fn from_path(path: impl AsRef<Path>) -> Result<(Self, Vec<Error>), Error> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let mut index_path = path.as_ref().to_path_buf();
        index_path.push("index.theme");
        let index_file = Ini::load_from_file(index_path.as_path())?;
        let theme_section = index_file.section(Some("Icon Theme".to_owned())).ok_or(
            CreateError::MissingIndexTheme {
                path: path_str.clone(),
            },
        )?;
        let name = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let display_name = theme_section
            .get("Name")
            .ok_or(CreateError::MissingProp {
                path: path_str.clone(),
                name: "Name",
            })?
            .clone();
        let comment = theme_section
            .get("Comment")
            .ok_or(CreateError::MissingProp {
                path: path_str.clone(),
                name: "Comment",
            })?
            .clone();
        let inherits = match theme_section.get("Inherits") {
            Some(value) => value.split(",").map(|s| s.to_owned()).collect(),
            None => Vec::new(),
        };
        let example = theme_section.get("Hidden").map(|s| s.to_owned());
        let hidden = parse_optional_prop("Hidden", path_str.clone(), theme_section, false)?;
        let (directories, errors): (Vec<_>, Vec<_>) = theme_section
            .get("Directories")
            .ok_or(CreateError::MissingProp {
                path: path_str.clone(),
                name: "Directories",
            })?
            .split(",")
            .map(|name| Directory::from(&name, path.as_ref(), &index_file))
            .partition(Result::is_ok);
        let mut errors: Vec<Error> = errors.into_iter().map(Result::unwrap_err).collect();
        let directories = directories
            .into_iter()
            .map(|item| {
                let (directory, errs) = item.unwrap();
                errors.extend(errs);
                directory
            })
            .collect();
        let (scaled_directories, errs) = match theme_section.get("ScaledDirectories") {
            Some(value) => value
                .split(",")
                .map(|name| Directory::from(&name, path.as_ref(), &index_file))
                .partition(Result::is_ok),
            None => (Vec::new(), Vec::new()),
        };
        let scaled_directories = scaled_directories
            .into_iter()
            .map(|item| {
                let (directory, errs) = item.unwrap();
                errors.extend(errs);
                directory
            })
            .collect();
        errors.extend::<Vec<Error>>(errs.into_iter().map(Result::unwrap_err).collect());

        Ok((
            IconTheme {
                name,
                display_name,
                comment,
                inherits,
                directories,
                scaled_directories,
                hidden,
                example,
            },
            errors,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Directory {
    path: PathBuf,
    icons: Vec<(String, String)>,
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
    pub fn from(
        name: &str,
        base_path: &Path,
        index_file: &ini::ini::Ini,
    ) -> Result<(Self, Vec<Error>), Error> {
        let path_str = base_path.to_string_lossy().into_owned();
        let dir_section =
            index_file
                .section(Some(name.to_owned()))
                .ok_or(CreateError::MissingDirSection {
                    path: path_str.clone(),
                    name: name.to_owned(),
                })?;
        let mut path = base_path.to_path_buf();
        path.push(&name);
        let size = dir_section
            .get("Size")
            .ok_or(CreateError::MissingDirProp {
                path: path_str.clone(),
                name: name.to_owned(),
                prop_name: "Size",
            })?
            .parse()
            .map_err(|_| CreateError::InvalidPropVal {
                name: "Size",
                path: path_str.clone(),
                value: dir_section.get("Size").unwrap().clone(),
            })?;
        let scale = parse_optional_prop("Scale", path_str.clone(), &dir_section, 1)?;
        let max_size = parse_optional_prop("MaxSize", path_str.clone(), &dir_section, size)?;
        let min_size = parse_optional_prop("MinSize", path_str.clone(), &dir_section, size)?;
        let threshold = parse_optional_prop("Threshold", path_str.clone(), &dir_section, 2)?;
        let type_ = parse_optional_prop(
            "Type",
            path_str.clone(),
            &dir_section,
            DirectoryType::Threshold,
        )?;
        let context = dir_section.get("Context").map(|s| s.to_owned());
        let mut icons = Vec::new();
        let mut errors = Vec::new();
        for entry in read_dir(&path)? {
            match entry {
                Ok(entry) => {
                    if entry.path().is_file() {
                        icons.push((
                            entry
                                .path()
                                .file_stem()
                                .ok_or(CreateError::MissingExt)?
                                .to_string_lossy()
                                .into_owned(),
                            entry
                                .path()
                                .extension()
                                .ok_or(CreateError::MissingExt)?
                                .to_string_lossy()
                                .into_owned(),
                        ))
                    }
                }
                Err(e) => errors.push(e.into()),
            }
        }

        Ok((
            Directory {
                path,
                icons,
                size,
                scale,
                context,
                type_,
                max_size,
                min_size,
                threshold,
            },
            errors,
        ))
    }
}

impl Default for Directory {
    fn default() -> Self {
        Directory {
            path: Default::default(),
            icons: Default::default(),
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DirectoryType {
    Fixed,
    Scalable,
    Threshold,
}

impl FromStr for DirectoryType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Fixed" => Ok(DirectoryType::Fixed),
            "Scalable" => Ok(DirectoryType::Scalable),
            "Threshold" => Ok(DirectoryType::Threshold),
            _ => Err(()),
        }
    }
}

fn parse_optional_prop<T: FromStr>(
    name: &'static str,
    path: String,
    map: &HashMap<String, String>,
    default: T,
) -> Result<T, Error> {
    match map.get(name) {
        Some(text) => Ok(text.parse().map_err(|_| CreateError::InvalidPropVal {
            name,
            path,
            value: text.to_owned(),
        })?),
        None => Ok(default),
    }
}
