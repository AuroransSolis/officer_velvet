use super::*;

pub fn load_gulag_sentences() -> Vec<GulagEntry> {
    // Create a collection to store the gulag entries in
    let mut gulags = Vec::new();
    let dir_iter = read_dir(GULAG_DIR).expect("Failed to read contents of gulags directory.");
    for item in dir_iter {
        if let Ok(dir_entry) = item {
            println!("Found file: {:?}", dir_entry.file_name());
            // Use the filename for the user ID and just chop '.gulag' off the end
            let mut user_id = dir_entry.file_name().into_string().unwrap();
            for _ in 0..6 {
                let _ = user_id.pop();
            }
            // Turn the string into a UserId
            let user_id = UserId(user_id.parse::<u64>()
                .expect("    Failed to parse filename as u64."));
            // Open the file for reading
            let mut file = File::open(dir_entry.path().as_path())
                .expect("    Failed to read file.");
            // Read the offset from the Unix Epoch for the end of the gulag sentence
            let offset = file.read_u64::<LittleEndian>()
                .expect("    Failed to read offset from file.");
            // Collect all the role IDs from the file
            let mut role_ids = Vec::new();
            while let Ok(role_id) = file.read_u64::<LittleEndian>() {
                role_ids.push(RoleId(role_id));
            }
            let path = dir_entry.path().as_path().to_owned();
            gulags.push(GulagEntry {
                file_path: path,
                user_id,
                previous_roles: role_ids,
                gulag_sentence: offset
            });
        }
    }
    gulags
}

// Filename: USER_ID.gulag
// File format:
// - offset from Unix Epoch for end of gulag sentence
// - role IDs
pub fn write_gulag_file(time: u64, user: UserId, message: &Message) -> Option<GulagEntry> {
    let path= format!("{}/{}.gulag", GULAG_DIR, user.0);
    let roles = if let Ok(member) = AXOLOTL_ARMADA_GID.member(user.0) {
        member.roles
    } else {
        let r = message.reply("Could not find Member for provided user ID.").unwrap();
        println!("    Failed to find member.");
        delete_message_after_delay(r, 10);
        return None;
    };
    let mut file = File::create(path.as_str()).expect("Failed to create new gulag file.");
    let offset_from_epoch = (SystemTime::now() + Duration::from_secs(time))
        .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let _ = file.write_u64::<LittleEndian>(offset_from_epoch)
        .expect("Failed to write epoch offset to file.");
    for &role_id in roles.iter() {
        let _ = file.write_u64::<LittleEndian>(role_id.0)
            .expect("Failed to write role ID to file.");
    }
    Some(GulagEntry {
        file_path: Path::new(path.as_str()).to_owned(),
        user_id: user,
        previous_roles: roles,
        gulag_sentence: offset_from_epoch
    })
}

pub fn start_gulag_sentences(gulag_role: RoleId, gulags: Vec<GulagEntry>) {
    for gulag in gulags {
        println!("Starting gulag handling for user: {}", gulag.user_id);
        // We're going to use a separate thread to handle waiting until the end of the gulag
        // sentence. It will wait until then and will have 25 attempts at editing the roles of a
        // given member before it gives up. This is so that it can attempt multiple times if there's
        // a bad connection, but it will also stop at some point in case the user has left by the
        // end of the gulag sentence. Then it will try to delete the gulag file.
        if let Ok(_) = AXOLOTL_ARMADA_GID
            .edit_member(gulag.user_id, |m| m.roles(vec![gulag_role])) {
            thread::spawn(move || {
                // Wait until the end of the gulag sentence
                let duration = (SystemTime::UNIX_EPOCH + Duration::from_secs(gulag.gulag_sentence))
                    .duration_since(SystemTime::now()).unwrap_or(Duration::from_secs(0));
                sleep(duration);
                // Try to add the roles back to the user
                let mut successfully_added_roles = false;
                'edit: for _ in 0..25 {
                    if let Ok(_) = AXOLOTL_ARMADA_GID
                        .edit_member(gulag.user_id, |m| m.roles(gulag.previous_roles.clone())) {
                        successfully_added_roles = true;
                        break 'edit;
                    }
                }
                if successfully_added_roles {
                    println!("Deleting persistent gulag file for user: {}", gulag.user_id);
                    remove_file(gulag.file_path)
                        .expect("    Failed to delete persistent gulag information.");
                } else {
                    println!("Failed to add roles back for user: {}", gulag.user_id);
                }
            });
        }
    }
}

pub fn start_new_gulag_sentence(context: &Context, message: &Message, gulag: GulagEntry) -> bool {
    // Pull the gulag role out of the cache. This is the main reason why we cached it in main.
    let gulag_role = if let Some(lock) = context.data.try_lock() {
        lock.get::<GulagRole>().unwrap().id
    } else {
        let r = message.reply("Cache was busy. Try again.").unwrap();
        println!("    Cache busy. Could not fetch gulag role.");
        delete_message_after_delay(r, 10);
        return false;
    };
    println!("    Attempting to start gulag for user: {}", gulag.user_id);
    // Attempt to set the user's roles to only the gulag role
    if let Ok(_) = AXOLOTL_ARMADA_GID
        .edit_member(gulag.user_id, |m| m.roles(vec![gulag_role])) {
        // If that was successful, do the same thing we did above. Spawn a thread that waits until
        // the gulag sentence is over and then tries to give the user's roles back.
        thread::spawn(move || {
            let duration = (SystemTime::UNIX_EPOCH + Duration::from_secs(gulag.gulag_sentence))
                .duration_since(SystemTime::now()).unwrap_or(Duration::from_secs(0));
            sleep(duration);
            let mut successfully_added_roles = false;
            'edit: for _ in 0..25 {
                if let Ok(_) = AXOLOTL_ARMADA_GID
                    .edit_member(gulag.user_id, |m| m.roles(gulag.previous_roles.clone())) {
                    successfully_added_roles = true;
                    break 'edit;
                }
            }
            if successfully_added_roles {
                println!("Deleting persistent gulag file for user: {}", gulag.user_id);
                remove_file(gulag.file_path)
                    .expect("    Failed to delete persistent gulag.");
            } else {
                println!("Failed to add roles back for user: {}", gulag.user_id);
            }
        });
        true
    } else {
        let r = message.reply("Failed to add gulag role and remove all others.").unwrap();
        println!("    Failed to remove all roles and add 'Prisoner' role.");
        delete_message_after_delay(r, 10);
        false
    }
}