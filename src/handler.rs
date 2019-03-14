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
    fn message(&self, context: Context, _message: Message) {
        println!("Begin handling message send.");
        let start = Instant::now();
        // Try to open the counter file
        if let Ok(mut file) = File::open(COUNTER_FILE) {
            // Try to read offset of file creation from the Unix Epoch
            if let Ok(offset_from_unix_epoch) = file.read_u64::<LittleEndian>() {
                let start_of_collection_period = SystemTime::UNIX_EPOCH
                    + Duration::from_secs(offset_from_unix_epoch);
                if let Ok(elapsed) = SystemTime::now().duration_since(start_of_collection_period) {
                    if elapsed.as_secs() > GATHERING_PERIOD {
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
                }
            }
            let num = file.read_u64::<LittleEndian>().expect("    Failed to read activity file.");
            let _  = File::create(COUNTER_FILE).expect("    Failed to clear counter file.");
            let _ = file.write_u64::<LittleEndian>(num + 1)
                .expect("    Failed to write new number to file.");
            println!("    Successfully handled message event.");
            println!("    Elapsed: {:?}", start.elapsed());
        }
    }
}