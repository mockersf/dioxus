use std::{process::exit, thread, time::Duration};

use dioxus_devtools::{DevserverMsg, connect, subsecond};

use test_hotpatching;

pub fn world() -> &'static str {
    "hello"
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
    let original_hello = test_hotpatching::hello();
    loop {
        let hello = subsecond::call(|| test_hotpatching::hello());
        let world = subsecond::call(|| world());
        println!("{} {}", hello, world);
        if hello != world {
            diffs += 1;
        } else {
            equals += 1;
        }
        if diffs > 5 {
            if original_hello == hello {
                // Hotpatching happened only in main.rs
                exit(3);
            }
            // hotpatching happened only in lib.rs
            exit(2);
        }
        if equals > 15 {
            if original_hello == hello {
                // hotpatching didn't happen at all
                exit(1);
            }
            // hotpatching of both main.rs and lib.rs successful!
            exit(0);
        }
        thread::sleep(Duration::from_secs(1));
    }
}
