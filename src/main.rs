use fuzzy_matcher::skim::fuzzy_match;
use launcher::scan::*;
use launcher::{self, App};
use rmp_serde as rmp;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;


const DB_PATH: &'static str = "apps.db";

fn main() {
    let db_path = Path::new(&DB_PATH);
    let apps: Vec<App> = if db_path.exists() {
        let mut apps_file = File::open(&db_path).unwrap();
        let mut buf = Vec::new();
        apps_file.read_to_end(&mut buf).unwrap();
        let mut de = rmp::Deserializer::new(&buf[..]);
        Deserialize::deserialize(&mut de).unwrap()
    } else {
        let desktop_files = desktop_files();
        let desktop_files = desktop_files.unwrap();
        let (apps, _errs) = parse_parse_entries(desktop_files);
        let mut buf = Vec::new();
        apps.serialize(&mut rmp::Serializer::new(&mut buf)).unwrap();
        let mut file = File::create("apps.db").unwrap();
        file.write_all(&buf).unwrap();
        apps
    };
    let mut app_list = apps
        .iter()
        .filter_map(|app| fuzzy_match(&app.name, "vs").map(|score| (app, score)))
        .collect::<Vec<(&App, i64)>>();
    app_list.sort_by(|left, right| right.1.cmp(&left.1));
    for (app, score) in app_list {
        println!("{}\t{}", app, score);
    }
}
