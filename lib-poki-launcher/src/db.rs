use log::*;
use std::cmp::Ordering;

use super::App;
use serde_derive::{Deserialize, Serialize};
use std::process;
use std::time::SystemTime;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppsDB {
    pub apps: Vec<App>,
    reference_time: f64,
    half_life: f32,
}

#[allow(dead_code)]
impl AppsDB {
    pub fn new(apps: Vec<App>) -> Self {
        AppsDB {
            apps,
            reference_time: current_time_secs(),
            half_life: 60.0 * 60.0 * 24.0 * 3.0,
        }
    }

    pub fn sort(&mut self) {
        self.apps.sort_unstable_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(Ordering::Less)
        });
    }

    fn secs_elapsed(&self) -> f32 {
        (current_time_secs() - self.reference_time) as f32
    }

    pub fn update_score(&mut self, uuid: &str, weight: f32) {
        let elapsed = self.secs_elapsed();
        self.apps
            .iter_mut()
            .find(|app| app.uuid == *uuid)
            .unwrap()
            .update_frecency(weight, elapsed, self.half_life);
    }

    pub fn merge(&mut self, mut apps_to_merge: Vec<App>) {
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