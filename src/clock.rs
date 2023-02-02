use std::time::{Duration, SystemTime};

fn adjust_system_time(offset: Duration) {
    let mut system_time = SystemTime::now();
    system_time = system_time.checked_add(offset).unwrap();
}
