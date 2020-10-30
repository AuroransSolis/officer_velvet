use crate::{DAY_AS_SECS, SHIT_CHANNEL, WEEK_AS_SECS};
use std::fs::File;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::{prelude::*, Utc, Weekday};

const REGINALD_SCHEDULER_FILE: &str = "reginalds-schedule";
const REGINALD_IS_HERE: &[u8] = include_bytes!("../reginald.png");
const HE_COMETH: &str = "HE COMETH.png";

fn is_tuesday() -> bool {
    let day = Utc::today().weekday();
    match day {
        Weekday::Tue => true,
        _ => false,
    }
}

fn he_visits() {
    let _ = SHIT_CHANNEL.send_files(vec![(REGINALD_IS_HERE, HE_COMETH)].into_iter(), |m| m);
}

// Reginald scheduler:
// First byte: bool (v)
// if v {
//     next eight bytes are for the next visit
// } else {
//     next eight bytes are the time of the last visit
// }
pub fn reginald_visits() {
    thread::spawn(|| {
        if let Err(_) = File::open(REGINALD_SCHEDULER_FILE) {
            let mut new_scheduler = File::create(REGINALD_SCHEDULER_FILE).unwrap();
            if is_tuesday() {
                new_scheduler.write_u8(0).unwrap();
                let next = (SystemTime::now() + Duration::from_secs(WEEK_AS_SECS * 2))
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                new_scheduler.write_u64::<LittleEndian>(next).unwrap();
                he_visits();
                thread::sleep(Duration::from_secs(WEEK_AS_SECS * 2));
            } else {
                new_scheduler.write_u8(1).unwrap();
                let days_until = 8 - Utc::today().weekday().num_days_from_monday() as u64;
                let next = (SystemTime::now() + Duration::from_secs(DAY_AS_SECS * days_until))
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                new_scheduler.write_u64::<LittleEndian>(next).unwrap();
                thread::sleep(Duration::from_secs(next));
            }
        }
        loop {
            let mut file = File::open(REGINALD_SCHEDULER_FILE).unwrap();
            let wait = file.read_u8().unwrap();
            if wait != 0 {
                let wait_until = file.read_u64::<LittleEndian>().unwrap();
                thread::sleep(Duration::from_secs(wait_until));
            }
            he_visits();
            let mut file = File::create(REGINALD_SCHEDULER_FILE).unwrap();
            file.write_u8(0).unwrap();
            let next = (SystemTime::now() + Duration::from_secs(WEEK_AS_SECS * 2))
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            file.write_u64::<LittleEndian>(next).unwrap();
            thread::sleep(Duration::from_secs(next));
        }
    });
}
