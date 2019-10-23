use linux_icon_theme::IconCache;

fn main() {
    let (icon_cache, errors) = IconCache::create().unwrap();
    println!("Creation done");
    // println!("{:#?}", icon_cache);
    // eprintln!("Errors: {:#?}", errors);
    let icon = icon_cache.get_icon("hicolor", "firefox", 128);
    println!("{:?}", icon);
}
