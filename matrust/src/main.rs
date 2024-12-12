use std::{
    env,
    error::Error,
    fmt,
    io::{self, Write},
    process::exit,
};

use anyhow::anyhow;
use matrix_sdk::{
    config::SyncSettings,
    ruma::events::room::member::StrippedRoomMemberEvent,
    ruma::{
        api::client::session::get_login_types::v3::{IdentityProvider, LoginType},
        events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
        {events::room::message::SyncRoomMessageEvent, user_id},
    },
    Client, Room, RoomState,
};
use tokio::time::{sleep, Duration};
use url::Url;

/// The initial device name when logging in with a device for the first time.
const INITIAL_DEVICE_DISPLAY_NAME: &str = "login client";

/// A simple program that adapts to the different login methods offered by a
/// Matrix homeserver.
///
/// Homeservers usually offer to login either via password, Single Sign-On (SSO)
/// or both.
/// use std::error::Error;

slint::include_modules!();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ui = AppWindow::new().expect("Failed to create AppWindow");
    ui.global::<LoginLogic>().on_login({
        //let ui_handle = ui.as_weak();
        move |server, username, password| {
            //let ui = ui_handle.unwrap()

            tokio::spawn(async move {
                let url = server.to_string();
                match login_and_sync_with_password(url, username.to_string(), password.to_string())
                    .await
                {
                    Ok(client) => {
                        // Handle successful login
                        println!("Login successful");
                        // You might want to update UI here if needed
                        // Now that we are logged in, we can sync and listen to new messages.
                        client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
                            println!("Received a message {:?}", ev);
                        });
                        client.add_event_handler(on_room_message);
                        client.add_event_handler(on_stripped_state_member);
                        // This will sync until an error happens or the program is killed.
                        if let Err(e) = client.sync(SyncSettings::default()).await {
                            eprintln!("Sync error: {}", e);
                        }
                        println!("Exiting");
                    }
                    Err(e) => {
                        // Handle login error
                        println!("Login failed: {}", e);
                    }
                }
            });
        }
    });

    ui.run()?;
    Ok(())
}

async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    println!("sono in stripped");
    if room_member.state_key != client.user_id().unwrap() {
        return;
    }

    tokio::spawn(async move {
        println!("Autojoining room {}", room.room_id());
        let mut delay = 2;

        while let Err(err) = room.join().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            eprintln!(
                "Failed to join room {} ({err:?}), retrying in {delay}s",
                room.room_id()
            );

            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                eprintln!("Can't join room {} ({err:?})", room.room_id());
                break;
            }
        }
        println!("Successfully joined room {}", room.room_id());
    });
}

async fn login_and_sync_with_password(
    homeserver_url: String,
    username: String,
    password: String,
) -> anyhow::Result<(Client)> {
    let homeserver_url = Url::parse(&homeserver_url)?;
    let client = Client::new(homeserver_url).await?;

    loop {
        match client
            .matrix_auth()
            .login_username(&username, &password)
            .initial_device_display_name(INITIAL_DEVICE_DISPLAY_NAME)
            .await
        {
            Ok(_) => {
                println!("Logged in as {username}");
                break;
            }
            Err(error) => {
                println!("Error logging in: {error}");
                println!("Please try again\n");
            }
        }
    }
    Ok(client)
}

/// Handle room messages by logging them.
async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    // We only want to listen to joined rooms.
    println!("In on_room_message");
    if room.state() != RoomState::Joined {
        return;
    }

    println!("In on_room_message2");
    // We only want to log text messages.
    let MessageType::Text(msgtype) = &event.content.msgtype else {
        return;
    };

    println!("In on_room_message3");
    let member = room
        .get_member(&event.sender)
        .await
        .expect("Couldn't get the room member")
        .expect("The room member doesn't exist");
    let name = member.name();

    println!("{name}: {}", msgtype.body);
}
