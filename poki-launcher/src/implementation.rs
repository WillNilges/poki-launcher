use super::interface::*;
use gtk::{Application, IconLookupFlags, IconTheme, IconThemeExt};
use lib_poki_launcher::prelude::*;
use std::path::Path;

const DB_PATH: &'static str = "apps.db";
const MAX_APPS_SHOWN: usize = 5;

pub struct AppsModel {
    emit: AppsModelEmitter,
    model: AppsModelList,
    list: Vec<App>,
    apps: AppsDB,
    selected_item: String,
}

impl AppsModelTrait for AppsModel {
    fn new(emit: AppsModelEmitter, model: AppsModelList) -> AppsModel {
        let _application =
            Application::new(Some("info.bengoldberg.poki_launcher"), Default::default())
                .expect("failed to initialize GTK application");
        let db_path = Path::new(&DB_PATH);
        let apps = if db_path.exists() {
            AppsDB::load(&DB_PATH).expect("Failed to load app db")
        } else {
            let apps = AppsDB::from_desktop_entries().unwrap();
            apps.save(&DB_PATH).expect("Faile to write db to disk");
            apps
        };

        AppsModel {
            emit,
            model,
            list: Vec::new(),
            apps,
            selected_item: String::new(),
        }
    }

    fn emit(&mut self) -> &mut AppsModelEmitter {
        &mut self.emit
    }

    fn row_count(&self) -> usize {
        self.list.len()
    }

    fn selected(&self) -> &str {
        &self.selected_item
    }

    fn set_selected(&mut self, value: String) {
        self.selected_item = value;
    }

    fn name(&self, index: usize) -> &str {
        if index < self.list.len() {
            &self.list[index].name
        } else {
            ""
        }
    }

    fn uuid(&self, index: usize) -> &str {
        if index < self.list.len() {
            &self.list[index].uuid
        } else {
            ""
        }
    }

    fn icon(&self, index: usize) -> &str {
        if index < self.list.len() {
            &self.list[index].icon
        } else {
            ""
        }
    }

    fn search(&mut self, text: String) {
        self.model.begin_reset_model();
        self.list = self.apps.get_ranked_list(&text, Some(MAX_APPS_SHOWN));
        if self.list.len() > 0 {
            self.selected_item = self.list[0].uuid.clone();
        } else {
            self.selected_item = String::new();
        }
        self.model.end_reset_model();
    }

    fn down(&mut self) {
        if self.list.len() <= 0 {
            return;
        }
        self.model.begin_reset_model();
        let (idx, _) = self
            .list
            .iter()
            .enumerate()
            .find(|(_, app)| app.uuid == self.selected_item)
            .unwrap();
        if idx >= self.list.len() - 1 {
            self.selected_item = self.list[self.list.len() - 1].uuid.clone();
        } else {
            self.selected_item = self.list[idx + 1].uuid.clone();
        }
        self.model.end_reset_model();
    }

    fn up(&mut self) {
        if self.list.len() <= 0 {
            return;
        }
        self.model.begin_reset_model();
        let (idx, _) = self
            .list
            .iter()
            .enumerate()
            .find(|(_, app)| app.uuid == self.selected_item)
            .unwrap();
        if idx == 0 {
            self.selected_item = self.list[0].uuid.clone();
        } else {
            self.selected_item = self.list[idx - 1].uuid.clone();
        }
        self.model.end_reset_model();
    }

    fn run(&mut self) {
        if self.list.len() <= 0 {
            return;
        }
        self.model.begin_reset_model();
        let app = self
            .list
            .iter()
            .find(|app| app.uuid == self.selected_item)
            .unwrap();
        // TODO Handle app run failures
        if let Err(err) = app.run() {
            eprintln!("Failed to execute app:\n{:#?}", err);
        }
        self.apps.update(app);
        self.apps.save(&DB_PATH).unwrap();
        self.list.clear();
        self.model.end_reset_model();
    }

    fn get_icon(&self, name: String) -> String {
        if Path::new(&name).is_absolute() {
            name
        } else {
            let theme = IconTheme::get_default().unwrap();
            let icon = match theme.lookup_icon(&name, 128, IconLookupFlags::empty()) {
                Some(icon) => icon,
                None => {
                    eprintln!("No icon found for {}", name);
                    return String::new();
                }
            };
            let path = (*icon.get_filename().unwrap().clone().to_string_lossy()).to_owned();
            path
        }
    }
}
