use std::{process::exit, thread, time::Duration};

use dioxus_devtools::{DevserverMsg, connect, subsecond};

use other_crate::SharedStruct;

pub fn world() -> SharedStruct {
    SharedStruct("hello")
}

fn main() {
    connect(|msg| {
        if let DevserverMsg::HotReload(hot_reload_msg) = msg {
            println!("-> DevserverMsg::HotReload");
            if let Some(jumptable) = hot_reload_msg.jump_table {
                println!("--> has jumptable ({})", jumptable.map.len());
                if hot_reload_msg.for_pid == Some(std::process::id()) {
                    println!("--> for this PID");
                    unsafe { subsecond::apply_patch(jumptable).unwrap() };
                }
            }
        }
    });
    let mut diffs = 0;
    let mut equals = 0;
    let original_hello = other_crate::other_hello().0;
    loop {
        let hello = subsecond::call(|| other_crate::other_hello().0);
        let world = subsecond::call(|| world().0);
        println!("{} {}", hello, world);
        if hello != world {
            diffs += 1;
        } else {
            equals += 1;
        }
        if diffs > 5 {
            if original_hello == hello {
                // Hotpatching happened only in the example code
                exit(3);
            }
            // hotpatching happened only in other crate
            exit(2);
        }
        if equals > 15 {
            if original_hello == hello {
                // hotpatching didn't happen at all
                exit(1);
            }
            // hotpatching of both example and other crate successful!
            exit(0);
        }
        thread::sleep(Duration::from_secs(1));
    }
}
