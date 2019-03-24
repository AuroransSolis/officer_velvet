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
    fn message(&self, _context: Context, _message: Message) {
        println!("Begin handling message send.");
        let start = Instant::now();
        // Try to open the counter file
        if let Ok(mut file) = File::open(COUNTER_FILE) {
            // Try to read offset of file creation from the Unix Epoch
            if let Ok(offset_from_unix_epoch) = file.read_u64::<LittleEndian>() {
                // Calculate start of collection period
                let start_of_collection_period = SystemTime::UNIX_EPOCH
                    + Duration::from_secs(offset_from_unix_epoch);
                // Try to calculate the amount of time since the specified start of collection
                if let Ok(elapsed) = SystemTime::now().duration_since(start_of_collection_period) {
                    if elapsed.as_secs() > GATHERING_PERIOD {
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
                    }
                } else {
                    println!("    SystemTime calculation error!");
                    print_elapsed_and_return!(start);
                }
            } else {
                println!("    Corrupted collection file!");
                print_elapsed_and_return!(start);
            }
            // Discard the offset (since we don't care about it anymore; not enough time has passed)
            // then get the count, clear the file, write the collection start and new count to the
            // file.
            let collection_start = file.read_u64::<LittleEndian>()
                .expect("    Failed to read offset from activity file.");
            let count = file.read_u64::<LittleEndian>().expect("    Failed to read activity file.");
            let _  = File::create(COUNTER_FILE).expect("    Failed to clear counter file.");
            let _ = file.write_u64::<LittleEndian>(collection_start)
                .expect("    Failed to write offset to file.");
            let _ = file.write_u64::<LittleEndian>(count + 1)
                .expect("    Failed to write new number to file.");
            println!("    Successfully handled message event.");
            println!("    Elapsed: {:?}", start.elapsed());
        }
    }
}