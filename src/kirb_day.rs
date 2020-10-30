use std::thread;
use std::time::Duration;

use chrono::Utc;
use {ANNOUNCEMENTS_CHANNEL, DAY_AS_SECS};

fn is_kirb_day() -> bool {
    format!("{}", Utc::today())
        .chars()
        .rev()
        .skip(3)
        .take(1)
        .next()
        .unwrap()
        == '8'
}

fn year_month_day() -> [u64; 3] {
    let mut date_string = format!("{}", Utc::today());
    for _ in 0..3 {
        date_string.pop();
    }
    let mut ymd = [0; 3];
    {
        let mut parts_iter = date_string.split('-');
        for i in 0..3 {
            ymd[i] = parts_iter.next().unwrap().parse::<u64>().unwrap();
        }
    }
    ymd
}

fn days_to_next_kirb_day() -> u64 {
    let ymd = year_month_day();
    let (year, month, day) = (ymd[0], ymd[1], ymd[2]);
    let days_in_month = match (year % 4 == 0, month) {
        (_, 1) => 31,     // Jan
        (false, 2) => 28, // Feb, not leap year
        (true, 2) => 29,  // Feb, leap year
        (_, 3) => 31,     // March
        (_, 4) => 30,     // April
        (_, 5) => 31,     // May
        (_, 6) => 30,     // June
        (_, 7) => 31,     // July
        (_, 8) => 31,     // August
        (_, 9) => 30,     // September
        (_, 10) => 31,    // October
        (_, 11) => 30,    // November
        (_, 12) => 31,    // December
        _ => unreachable!(),
    };
    days_in_month - day + 8
}

fn send_kirb_day_reminder() {
    ANNOUNCEMENTS_CHANNEL.send_message(|m| {
        m.content(
            "@everyone it's Kirb day, fucknuggets. Get \
        drawin' or it's gulag time.\n\n*Faint sounds of forced labor in the background*",
        )
    });
}

pub fn kirb_day_task() {
    thread::spawn(|| {
        if !is_kirb_day() {
            thread::sleep(Duration::from_secs(days_to_next_kirb_day() * DAY_AS_SECS));
        }
        loop {
            send_kirb_day_reminder();
            thread::sleep(Duration::from_secs(days_to_next_kirb_day() * DAY_AS_SECS));
        }
    });
}
