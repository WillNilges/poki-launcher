use super::interface::*;
use dbus::tree::*;
use dbus::*;
use failure::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start_dbus_server(
    emit: Mutex<AppsModelEmitter>,
    visible: Arc<AtomicBool>,
) -> Result<(), Error> {
    thread::spawn(move || {
        let conn = Connection::get_private(BusType::Session).unwrap();
        conn.register_name(
            "info.bengoldberg.poki_launcher",
            NameFlag::ReplaceExisting as u32,
        )
        .unwrap();
        let factory = Factory::new_fn::<()>();
        let tree = factory.tree(()).add(
            factory.object_path("/show", ()).introspectable().add(
                factory
                    .interface("info.bengoldberg.poki_launcher", ())
                    .add_m(factory.method("show", (), move |m| {
                        println!("Recieved show");
                        visible.store(true, Ordering::Relaxed);
                        emit.lock().unwrap().new_data_ready();
                        Ok(vec![m.msg.method_return()])
                    })),
            ),
        );
        tree.set_registered(&conn, true).unwrap();
        conn.add_handler(tree);
        println!("DBus starting");
        loop {
            conn.incoming(1000).next();
        }
    });
    Ok(())
}
