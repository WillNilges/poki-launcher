mod dbus_server;
mod implementation;
pub mod interface {
    include!(concat!(env!("OUT_DIR"), "/src/interface.rs"));
}

extern "C" {
    fn main_cpp(app: *const ::std::os::raw::c_char);
}

use dbus::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "poki-launcher", about = "An application launcher")]
struct Opt {
    #[structopt(short = "s", long = "show")]
    show: bool,
}

fn start_app() {
    use std::ffi::CString;
    let app_name = ::std::env::args().next().unwrap();
    let app_name = CString::new(app_name).unwrap();
    unsafe {
        main_cpp(app_name.as_ptr());
    }
}

fn show_app() {
    let conn = Connection::get_private(BusType::Session).unwrap();
    let obj = conn.with_path("info.bengoldberg.poki_launcher", "/show", 1000);
    let interface = Interface::new("info.bengoldberg.poki_launcher").unwrap();
    let member = Member::new("show").unwrap();
    obj.method_call_with_args(&interface, &member, |_| {})
        .unwrap();
}

fn main() {
    let opt = Opt::from_args();
    if opt.show {
        show_app();
    } else {
        start_app();
    }
}
