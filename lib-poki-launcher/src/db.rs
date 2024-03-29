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
use log::*;
use std::cmp::Ordering;

use super::App;
use failure::{Error, Fail};
use fuzzy_matcher::skim::fuzzy_match;
use rmp_serde as rmp;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write as _;
use std::path::Path;
use std::process;
use std::time::SystemTime;

/// An apps database.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppsDB {
    /// The list of apps.
    pub apps: Vec<App>,
    /// The reference time used in the ranking calculations.
    reference_time: f64,
    /// The half life of the app launches
    half_life: f32,
}

#[allow(dead_code)]
impl AppsDB {
    /// Create a new app.
    pub fn new(apps: Vec<App>) -> Self {
        AppsDB {
            apps,
            reference_time: current_time_secs(),
            // Half life of 3 days
            half_life: 60.0 * 60.0 * 24.0 * 3.0,
        }
    }

    /// Load database file.
    ///
    /// # Arguments
    ///
    /// * `path` - Location of the database file
    pub fn load(path: impl AsRef<Path>) -> Result<AppsDB, Error> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        Ok(
            rmp::from_read(File::open(&path).map_err(|e| AppDBError::FileOpen {
                file: path_str.clone(),
                err: e.into(),
            })?)
            .map_err(|e| AppDBError::ParseDB {
                file: path_str.clone(),
                err: e.into(),
            })?,
        )
    }

    /// Save database file.
    ///
    /// # Arguments
    ///
    /// * `path` - Location of the database file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let buf = rmp::to_vec(&self).expect("Failed to encode apps db");
        let mut file = File::create(&path).map_err(|e| AppDBError::FileCreate {
            file: path_str.clone(),
            err: e.into(),
        })?;
        file.write_all(&buf).map_err(|e| AppDBError::FileWrite {
            file: path_str.clone(),
            err: e.into(),
        })?;
        Ok(())
    }

    /// Get the apps in rank order for a given search string.
    ///
    /// This ranks the apps both by frecency score and fuzzy search.
    // TODO Remove num_items
    pub fn get_ranked_list(&self, search: &str, num_items: Option<usize>) -> Vec<App> {
        let mut app_list = self
            .apps
            .iter()
            .filter_map(|app| match fuzzy_match(&app.name, &search) {
                Some(score) if score > 0 => {
                    let mut app = app.clone();
                    app.score += score as f32;
                    Some(app)
                }
                _ => None,
            })
            .collect::<Vec<App>>();
        app_list.sort_by(|left, right| right.score.partial_cmp(&left.score).unwrap());
        if let Some(n) = num_items {
            app_list = app_list.into_iter().take(n).collect();
        }
        app_list
    }

    /// Increment to score for app `to_update` by 1 launch.
    pub fn update(&mut self, to_update: &App) {
        self.update_score(&to_update.uuid, 1.0);
    }

    /// Sort the apps database by score.
    pub fn sort(&mut self) {
        self.apps.sort_unstable_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(Ordering::Less)
        });
    }

    /// Seconds elapsed since the reference time.
    fn secs_elapsed(&self) -> f32 {
        (current_time_secs() - self.reference_time) as f32
    }

    /// Update the score of an app.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The uuid of the app to update.
    /// * `weight` - The amount to update to score by.
    pub fn update_score(&mut self, uuid: &str, weight: f32) {
        let elapsed = self.secs_elapsed();
        self.apps
            .iter_mut()
            .find(|app| app.uuid == *uuid)
            .unwrap()
            .update_frecency(weight, elapsed, self.half_life);
    }

    /// Merge the apps from a re-scan into the database.
    ///
    /// * Apps in `self` that are not in `apps_to_merge` will be removed from `self`
    /// * Apps in `apps_to_merge` not in `self` will be added to `self`
    pub fn merge_new_entries(&mut self, mut apps_to_merge: Vec<App>) {
        let apps = std::mem::replace(&mut self.apps, Vec::new());
        self.apps = apps
            .into_iter()
            .filter(|app| apps_to_merge.contains(app))
            .collect();
        apps_to_merge = apps_to_merge
            .into_iter()
            .filter(|app| !self.apps.contains(app))
            .collect();
        self.apps.extend(apps_to_merge);
    }
}

#[allow(dead_code)]
impl App {
    fn get_frecency(&self, elapsed: f32, half_life: f32) -> f32 {
        self.score / 2.0f32.powf(elapsed / half_life)
    }

    fn set_frecency(&mut self, new: f32, elapsed: f32, half_life: f32) {
        self.score = new * 2.0f32.powf(elapsed / half_life);
    }

    fn update_frecency(&mut self, weight: f32, elapsed: f32, half_life: f32) {
        self.set_frecency(
            self.get_frecency(elapsed, half_life) + weight,
            elapsed,
            half_life,
        );
    }
}

/// Return the current time in seconds as a float
#[allow(dead_code)]
pub fn current_time_secs() -> f64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => (u128::from(n.as_secs()) * 1000 + u128::from(n.subsec_millis())) as f64 / 1000.0,
        Err(e) => {
            error!("invalid system time: {}", e);
            process::exit(1);
        }
    }
}

#[derive(Debug, Fail)]
pub enum AppDBError {
    #[fail(display = "Failed to open apps database file {}: {}", file, err)]
    FileOpen { file: String, err: Error },
    #[fail(display = "Failed to create apps database file {}: {}", file, err)]
    FileCreate { file: String, err: Error },
    #[fail(display = "Failed to write to apps database file {}: {}", file, err)]
    FileWrite { file: String, err: Error },
    #[fail(display = "Couldn't parse apps database file {}: {}", file, err)]
    ParseDB { file: String, err: Error },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_new_entries_identical() {
        let apps = vec![
            App::new(
                "Test1".to_owned(),
                "icon".to_owned(),
                "/bin/test".to_owned(),
            ),
            App::new(
                "Test2".to_owned(),
                "icon".to_owned(),
                "/bin/test".to_owned(),
            ),
        ];
        let mut apps_db = AppsDB::new(apps.clone());
        apps_db.merge_new_entries(apps.clone());
        assert_eq!(apps, apps_db.apps);
    }

    #[test]
    fn merge_new_entries_remove() {
        let mut apps = vec![
            App::new(
                "Test1".to_owned(),
                "icon".to_owned(),
                "/bin/test".to_owned(),
            ),
            App::new(
                "Test2".to_owned(),
                "icon".to_owned(),
                "/bin/test".to_owned(),
            ),
        ];
        let mut apps_db = AppsDB::new(apps.clone());
        apps.remove(0);
        apps_db.merge_new_entries(apps.clone());
        assert_eq!(apps, apps_db.apps);
    }

    #[test]
    fn merge_new_entries_add() {
        let mut apps = vec![App::new(
            "Test1".to_owned(),
            "icon".to_owned(),
            "/bin/test".to_owned(),
        )];
        let mut apps_db = AppsDB::new(apps.clone());
        apps.push(App::new(
            "Test2".to_owned(),
            "icon".to_owned(),
            "/bin/test".to_owned(),
        ));
        apps_db.merge_new_entries(apps.clone());
        assert_eq!(apps, apps_db.apps);
    }
}
