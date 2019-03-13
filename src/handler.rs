use super::*;

macro_rules! print_elapsed_and_return {
    ($instant:ident) => {{
        println!("    Elapsed: {:?}", $instant.elapsed());
        return;
    }}
}

pub struct Handler;

impl EventHandler for Handler {
    fn message(&self, context: Context, _message: Message) {
        let foo = context.data.try_lock().unwrap().get::<CachedPartialGuild>();
        println!("Begin handling message send.");
        let start = Instant::now();
        if let Ok(mut file) = File::open(COUNTER_FILE) {
            if let Ok(metadata) = file.metadata() {
                if let Ok(last_edited) = metadata.modified() {
                    if let Ok(elapsed) = last_edited.elapsed() {
                        if elapsed.as_secs() > GATHERING_PERIOD {
                            println!("    Gathering period has ended. Clearing file.");
                            let mut file = File::create(COUNTER_FILE)
                                .expect("    Failed to clear counter file after end of gathering \
                                    period.");
                            let _ = file.write_u64::<LittleEndian>(1)
                                .expect("    Failed to write 1 to new file.");
                            println!("    Successfully cleared and wrote 1 to new counter file.");
                            print_elapsed_and_return!(start);
                        }
                    } else {
                        println!("    Failed to get time elapsed since last edit.");
                        print_elapsed_and_return!(start);
                    }
                } else {
                    println!("    Failed to get SystemTime for last edit time.");
                    print_elapsed_and_return!(start);
                }
            } else {
                println!("    Failed to get counter file metadata.");
                print_elapsed_and_return!(start);
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