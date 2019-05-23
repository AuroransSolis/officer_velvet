use rand::{thread_rng, seq::SliceRandom};
use std::time::Instant;

command!(Anagram(_context, message) {
    println!("Start handling anagram command.");
    let start = Instant::now();
    let mut rng = thread_rng();
    // "=>anagram " = 10 chars
    let mut to_scramble = message.content.chars().skip(10).collect::<Vec<char>>();
    let start_msg = to_scramble.iter().collect::<String>();
    to_scramble.shuffle(&mut rng);
    let end_msg = String::from_utf8(to_scramble.into_iter().map(|c| c as u8).collect::<Vec<u8>>())
        .unwrap();
    let msg = format!("Hey guys, did you know that \"{}\" is an anagram of \"{}?\"", end_msg,
        start_msg);
    let _ = message.delete();
    let _ = message.channel_id.send_message(|m| m.content(msg.as_str()));
    println!("    Elapsed: {:?}", start.elapsed());
});