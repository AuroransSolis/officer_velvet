#![allow(clippy::unreadable_literal)]

use crate::{
    gulag::GulagApp,
    misc::{escape_formatting, get_help_msg, is_administrator},
    release::ReleaseSearchCriteriumApp,
    tasks::{
        date_conditional_task::DateConditionalTask, message::MessageType,
        periodic_task::CreatePeriodicTask, task::Task, CreateTask,
    },
    EMBED_COLOUR, FOOTER_TEXT,
};
use chrono::{Duration, Utc};
use clap::CommandFactory;
use lazy_static::lazy_static;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use std::time::Instant;

lazy_static! {
    static ref HELP_HELP_MSG_NONADMIN: String = "\
        The `=>help` command can be used on its own for a short description of available commands, \
        or with any of the following command names to get more information on how to use them.\n\
        - `help`\n\
        - `anagram`\n\
        - `source`\n\
    "
    .to_string();
    static ref HELP_HELP_MSG_ADMIN: String = format!(
        "\
            {}\n\
            - `current_gulags`\n\
            - `gulag`\n\
            - `create_task`\
        ",
        HELP_HELP_MSG_NONADMIN.as_str(),
    );
    pub static ref GULAG_HELP_MSG: String = get_help_msg(GulagApp::command());
    pub static ref CREATE_TASK_HELP_MSG: String = {
        let mut string = get_help_msg(CreateTask::command());
        string.push_str(get_help_msg(DateConditionalTask::command()).as_str());
        string.push_str(get_help_msg(CreatePeriodicTask::command()).as_str());
        string
    };
    pub static ref RELEASE_HELP_MSG: String = get_help_msg(ReleaseSearchCriteriumApp::command());
    pub static ref CREATE_TASK_EXAMPLE: String = {
        format!(
            "\
            ```\n\
            =>create_task date_conditional_task --time {time_now} --dom 20 --moy 4\n\
            `\u{200b}`\u{200b}`json\n\
            {example_dct_json}\n\
            `\u{200b}`\u{200b}`\n\
            ```This creates a task that executes at {time_now} on the 20th day of the 4th month of \
            each year. The message 'haha yes' will be sent to channel ID 549647427397877822 \
            every time that date and time occurs.\n\
            ```\n\
            =>create_task periodic_task -s 1 -m 33 -h 7 -d 4 -w 2 --start {tomorrow}\n\
            `\u{200b}`\u{200b}`json\n\
            {example_pt_json}\n\
            `\u{200b}`\u{200b}`\n\
            ```This creates a task that executes every two weeks, four days, 7 hours, 33 minutes, \
            and 1 second. The bot's nickname will be changed to the new name and the avatar to the \
            image in the specified filename at that time.\
            ",
            time_now = Utc::now().time().format("%H:%M:%S"),
            example_dct_json = serde_json::to_string_pretty(&Task::SendMessage {
                send_to: 549647427397877822.into(),
                message: MessageType::Plain {
                    content: "haha yes".into()
                },
                upload_file: None,
            })
            .unwrap(),
            tomorrow = Utc::now()
                .checked_add_signed(Duration::days(1))
                .unwrap()
                .naive_utc()
                .date(),
            example_pt_json = serde_json::to_string_pretty(&Task::UpdateAppearance {
                new_name: "Totally Not Officer Velvet".into(),
                new_icon_filename: "disguise.png".into(),
            })
            .unwrap(),
        )
    };
}

macro_rules! def_help_lists {
    (
        all {
            $(
                {
                    $cmd:literal,
                    $short_desc:literal,
                    $long_help:expr,
                    $example:expr,
                },
            )+
        }
        admin {
            $(
                {
                    $admin_cmd:literal,
                    $admin_short_desc:literal,
                    $admin_long_help:expr,
                    $admin_example:expr,
                },
            )+
        }
    ) => {
        lazy_static! {
            static ref NONADMIN_HELP_INFO: Vec<[String; 4]> = vec![
                $(
                    [$cmd.into(), $short_desc.into(), $long_help, $example]
                ),+
            ];
            static ref ADMIN_HELP_INFO: Vec<[String; 4]> = vec![
                $(
                    [$cmd.into(), $short_desc.into(), $long_help, $example]
                ),+,
                $(
                    [$admin_cmd.into(), $admin_short_desc.into(), $admin_long_help, $admin_example]
                ),+
            ];
        }
    }
}

def_help_lists! {
    all {
        {
            "help",
            "Get short descriptions or help for a specific command.",
            "\
                Use this command with no arguments to get a list of commands available to you, or \
                pass in the name of a command to get more information on how to use it.\
            ".into(),
            "`=>help`\n`=>help anagram`".into(),
        },
        {
            "anagram",
            "Scrambles the rest of a message.",
            "Takes all of the input to a message after the space and scrambles it using RNG.".into(),
            "`=>anagram Here's an example usage!`".into(),
        },
        {
            "source",
            "Sends a link to my code repository.",
            "Sends a link to my code repository.".into(),
            "`=>source`".into(),
        },
    }
    admin {
        {
            "current_gulags",
            "Shows a list of the current gulag sentences.",
            "Sends an embed message showing all currently gulagged users and their release dates.".into(),
            "`=>current_gulags`".into(),
        },
        {
            "gulag",
            "Sends naughty boys to gulag to be educated and turned into girls.",
            GULAG_HELP_MSG.clone(),
            "\
                `=>gulag --user @some_user -s 1 -m 2 -h 3 -d 4 -w 5`\n\
                The above gulags the user `@some_user` for one second, two minutes, three \
                hours, four days, and five weeks. Note that `-s` could be replaced with \
                `--secs`, `-m` with `--mins`, and so on.\
            ".into(),
        },
        {
            "create_task",
            "Gives me something to do other than work prisoners to death.",
            CREATE_TASK_HELP_MSG.clone(),
            CREATE_TASK_EXAMPLE.clone(),
        },
        {
            "release",
            "Release a prisoner.",
            RELEASE_HELP_MSG.clone(),
            "\
                `=>release --user @some_user`\n\
                This releases the user @some_user from gulag.\n\n\
                `=>release --index N`\n\
                Searches through the task list and ends the `N`th gulag sentence.
            ".into(),
        },
        {
            "list_tasks",
            "Lists the tasks currently in the list.",
            "No arguments are expected. Should only be called as `=>list_tasks`.".into(),
            String::new(),
        },
    }
}

#[command]
pub async fn help(ctx: &Context, message: &Message) -> CommandResult {
    println!("HL | Start handling help command.");
    let start = Instant::now();
    let trimmed_content = message.content.trim_start_matches("=>help").trim();
    let icon_url = ctx.http.get_current_user().await?.avatar_url().unwrap();
    println!("HL | Got current avatar URL.");
    let help_list = if is_administrator(&ctx.http, ctx.data.read().await, message).await? {
        &ADMIN_HELP_INFO[..]
    } else {
        &NONADMIN_HELP_INFO[..]
    };
    println!("HL | Got help list.");
    let _ =
        message
            .channel_id
            .send_message(&ctx.http, |msg| {
                msg.embed(|embed| {
                    embed
                        .title("Okay, fine...")
                        .description("...I guess I can help you with that.")
                        .colour(EMBED_COLOUR)
                        .footer(|footer| footer.text(FOOTER_TEXT).icon_url(icon_url));
                    println!("HL | Constructed base embed.");
                    if trimmed_content.is_empty() {
                        println!("HL | User requested general help.");
                        embed.fields(help_list.iter().map(|[name, short_desc, ..]| {
                            (name.as_str(), short_desc.as_str(), false)
                        }))
                    } else if let Some([.., long_help, example]) = help_list
                        .iter()
                        .find(|[name, ..]| name.as_str() == trimmed_content)
                    {
                        println!("HL | User requested help with '{}'", trimmed_content,);
                        embed.fields(vec![
                            ("Command information", long_help.as_str(), false),
                            ("Usage", example.as_str(), false),
                        ])
                    } else {
                        println!("HL | User requested help for unknown command.");
                        embed.field(
                            "Excuse me what",
                            format!(
                                "\
                                No clue what\n\
                                ```{}```is. Make sure you entered a valid command name.\
                            ",
                                escape_formatting(trimmed_content),
                            ),
                            false,
                        )
                    }
                })
            })
            .await?;
    println!("HL | Elapsed: {:?}", start.elapsed());
    Ok(())
}
