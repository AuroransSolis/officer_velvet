use super::*;

// Convenience
macro_rules! print_elapsed_and_return {
    ($instant:ident) => {{
        println!("    Elapsed: {:?}", $instant.elapsed());
        return;
    }}
}

pub struct Handler;

impl EventHandler for Handler {
    fn message(&self, _context: Context, message: Message) {
        if message.author.id == BOT_UID {
            return;
        }
        println!("Begin handling message send.");
        let start = Instant::now();
        // Try to open the counter file
        let (offset_from_unix_epoch, count) = if let Ok(mut file) = File::open(COUNTER_FILE) {
            let offset_from_unix_epoch = file.read_u64::<LittleEndian>()
                .expect("Failed to read offset from epoch.");
            let count = file.read_u64::<LittleEndian>()
                .expect("Failed to read current collection period count.");
            if SystemTime::now().duration_since(SystemTime::UNIX_EPOCH
                + Duration::from_secs(offset_from_unix_epoch))
                .unwrap().as_secs() > GATHERING_PERIOD {
                // Open the file, read the number of messages that were sent, and send it to
                // Crak.
                let mut file = File::open(COUNTER_FILE).unwrap();
                let num_messages = file.read_u64::<LittleEndian>().unwrap();
                let crak = match CRAK_UID.to_user() {
                    Ok(user) => user,
                    Err(_) => return
                };
                let crak_msg = format!("{} messages were sent in the last gathering \
                            period.", num_messages);
                drop(crak.direct_message(|m| m.content(crak_msg.as_str())));
                // Then clear the file and write the new start of collection and 1.
                println!("    Gathering period has ended. Clearing file.");
                let mut file = File::create(COUNTER_FILE)
                    .expect("    Failed to clear counter file after end of gathering \
                                    period.");
                let _ = file.write_u64::<LittleEndian>(SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
                let _ = file.write_u64::<LittleEndian>(1)
                    .expect("    Failed to write 1 to new file.");
                println!("    Successfully cleared and wrote time and 1 to new counter \
                            file.");
                print_elapsed_and_return!(start);
            } else {
                (offset_from_unix_epoch, count)
            }
        } else {
            return;
        };
        // Clear the counter file and write the start of the collection period back along
        // with the new count.
        let mut file  = File::create(COUNTER_FILE).expect("    Failed to clear counter file.");
        let _ = file.write_u64::<LittleEndian>(offset_from_unix_epoch)
            .expect("    Failed to write offset to file.");
        let _ = file.write_u64::<LittleEndian>(count + 1)
            .expect("    Failed to write new number to file.");
        println!("    Successfully handled message event.");
        println!("    Elapsed: {:?}", start.elapsed());
    }
}