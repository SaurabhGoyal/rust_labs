use std::{
    io::{self, Write as _},
    thread,
    time::Duration,
};

use sysinfo;

fn main() {
    loop {
        println!("{}", get_info());
        thread::sleep(Duration::from_secs(1));
        clear_screen();
    }
}

fn clear_screen() {
    print!("{}[2J", 27 as char); // ANSI escape code to clear the screen
    print!("{}[1;1H", 27 as char); // ANSI escape code to move the cursor to the top-left corner
    io::stdout().flush().unwrap(); // Flush stdout to ensure screen is cleared immediately
}

fn get_info() -> String {
    let mut info: String = String::new();
    let sys = sysinfo::System::new_all();
    info.push_str(&format!("System -\n{:?}\n", sys));

    // for proc in info.processes() {
    //     println!("Process -\n{:?}\n", proc);
    // }

    // for disk in &sysinfo::Disks::new_with_refreshed_list() {
    //     info.push_str(&format!("Disk -\n{:?}\n", disk));
    // }

    // for network in &sysinfo::Networks::new_with_refreshed_list() {
    //     info.push_str(&format!("Network -\n{:?}\n", network));
    // }

    // for comp in &sysinfo::Components::new_with_refreshed_list() {
    //     info.push_str(&format!("Component -\n{:?}\n", comp));
    // }
    info
}
