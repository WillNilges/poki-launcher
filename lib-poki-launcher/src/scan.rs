/***
 * This file is part of Poki Launcher.
 *
 * Poki Launcher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Poki Launcher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Poki Launcher.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::db::AppsDB;
use crate::desktop_entry::{parse_desktop_file, EntryParseError};
use crate::App;
use failure::{Error, Fail};
use std::fs::read_dir;
use std::path::PathBuf;

/// An error from scanning for desktop entries.
#[derive(Debug, Fail)]
pub enum ScanError {
    /// Failed to scan the directory for some reason (ex. it doesn't exist).
    #[fail(
        display = "Failed to scan directory {} for desktop entries: {}",
        dir, err
    )]
    ScanDirectory { dir: String, err: Error },
    /// Paring the entry failed.
    #[fail(display = "Parse error: {}", err)]
    ParseEntry { err: EntryParseError },
    /// Path expansion failed.
    #[fail(display = "Failed to expand path {}: {}", path, err)]
    PathExpand { path: String, err: Error },
}

/// Get a list of desktop entries from a list of directories to search.
pub fn desktop_entires(paths: &[String]) -> (Vec<PathBuf>, Vec<Error>) {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for loc in paths {
        let expanded = match shellexpand::full(&loc) {
            Ok(path) => path,
            Err(e) => {
                errors.push(
                    ScanError::PathExpand {
                        path: loc.clone(),
                        err: e.into(),
                    }
                    .into(),
                );
                continue;
            }
        };
        match read_dir(&*expanded) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if entry.file_name().to_str().unwrap().contains(".desktop") {
                                files.push(entry.path())
                            }
                        }
                        Err(e) => {
                            errors.push(
                                ScanError::ScanDirectory {
                                    dir: loc.clone(),
                                    err: e.into(),
                                }
                                .into(),
                            );
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(
                    ScanError::ScanDirectory {
                        dir: loc.clone(),
                        err: e.into(),
                    }
                    .into(),
                );
            }
        }
    }
    (files, errors)
}

/// Get a list of apps for a list of paths to search.
pub fn scan_desktop_entries(paths: &[String]) -> (Vec<App>, Vec<Error>) {
    let (entries, mut errors) = desktop_entires(&paths);
    let (apps, errs): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .map(|path| parse_desktop_file(&path))
        .partition(Result::is_ok);
    let mut apps: Vec<_> = apps
        .into_iter()
        .map(Result::unwrap)
        .filter_map(|x| x)
        .collect();
    apps.sort_unstable();
    apps.dedup();
    errors.extend(errs.into_iter().map(Result::unwrap_err).collect::<Vec<_>>());
    (apps, errors)
}

impl AppsDB {
    /// Create an `AppsDB` from the desktop entries.
    ///
    /// # Arguments
    ///
    /// * `paths` - A list of paths to desktop entries.
    pub fn from_desktop_entries(paths: &[String]) -> (AppsDB, Vec<Error>) {
        let (apps, errors) = scan_desktop_entries(paths);
        (AppsDB::new(apps), errors)
    }

    /// Update self with new desktop entries.
    ///
    /// Scan the desktop entries again then merge the new list
    /// into self with `AppsDB.merge`.
    ///
    /// # Arguments
    ///
    /// * `paths` - A list of paths to desktop entries.
    pub fn rescan_desktop_entries(&mut self, paths: &[String]) -> Vec<Error> {
        let (apps, errors) = scan_desktop_entries(paths);
        self.merge_new_entries(apps);
        errors
    }
}
