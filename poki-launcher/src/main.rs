mod implementation;
pub mod interface {
    include!(concat!(env!("OUT_DIR"), "/src/interface.rs"));
}

use poki_launcher_notifier as notifier;

extern "C" {
    fn main_cpp(app: *const std::os::raw::c_char);
}

fn main() {
    if notifier::is_running() {
        if let Err(e) = notifier::notify() {
            eprintln!("{}", e);
            start_ui();
        }
    } else {
        start_ui();
    }
}

fn start_ui() {
    use std::ffi::CString;
    let app_name = std::env::args().next().unwrap();
    let app_name = CString::new(app_name).unwrap();
    unsafe {
        main_cpp(app_name.as_ptr());
    }
}
