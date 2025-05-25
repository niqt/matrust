use anyhow::anyhow;
use matrix_sdk::{
    Client, Room, RoomState,
    config::SyncSettings,
    ruma::events::room::member::StrippedRoomMemberEvent,
    ruma::{
        api::client::session::get_login_types::v3::{IdentityProvider, LoginType},
        events::room::message::{
            MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
        },
        {events::room::message::SyncRoomMessageEvent, user_id},
    },
};
use slint::{Model, ModelRc, PlatformError, VecModel};
use std::sync::Once;
use std::{borrow::BorrowMut, sync::Arc};
use std::{
    env,
    error::Error,
    fmt,
    io::{self, Write},
    process::exit,
};
//use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use slint::Weak;
use std::cell::OnceCell;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use tokio::time::{Duration, sleep};
use url::Url;

//use lazy_static::lazy_static;
/// The initial device name when logging in with a device for the first time.
const INITIAL_DEVICE_DISPLAY_NAME: &str = "login client";

/// A simple program that adapts to the different login methods offered by a
/// Matrix homeserver.
///
/// Homeservers usually offer to login either via password, Single Sign-On (SSO)
/// or both.
/// use std::error::Error;

slint::include_modules!();

static UI_HANDLE: Lazy<Mutex<Option<Weak<AppWindow>>>> = Lazy::new(|| Mutex::new(None));

fn with_ui<F>(f: F)
where
    F: FnOnce(&AppWindow),
{
    if let Some(ui_weak) = UI_HANDLE.lock().unwrap().as_ref() {
        if let Some(ui) = ui_weak.upgrade() {
            f(&ui);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ui = AppWindow::new().unwrap();
    *UI_HANDLE.lock().unwrap() = Some(ui.as_weak());

    let weak = ui.as_weak();

    ui.global::<LoginLogic>().on_login({
        let weak = weak.clone();
        move |server, username, password| {
            let weak = weak.clone();
            tokio::spawn(async move {
                let url = server.to_string();
                match login_and_sync_with_password(url, username.to_string(), password.to_string())
                    .await
                {
                    Ok(client) => {
                        println!("Login successful");
                        slint::invoke_from_event_loop(move || {
                            let ui = weak.upgrade().expect("Failed to upgrade weak reference");
                            ui.set_logged(true);
                        })
                        .expect("Failed to invoke from event loop");
                        // Set up event handlers
                        client.add_event_handler(on_room_message);
                        client.add_event_handler(on_stripped_state_member);
                        // Start syncing
                        match client.sync(SyncSettings::default()).await {
                            Ok(_) => println!("Sync completed successfully"),
                            Err(e) => eprintln!("Sync error: {}", e),
                        }
                    }
                    Err(e) => {
                        let error_message = format!("Login failed: {}", e);
                        eprintln!("{}", error_message);
                        slint::invoke_from_event_loop(move || {
                            let ui = weak.upgrade().expect("Failed to upgrade weak reference");
                            //ui.invoke_login_error(error_message);
                        })
                        .expect("Failed to invoke from event loop");
                    }
                }
            });
        }
    });
    ui.run();
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

    // We only want to log text messages.
    let MessageType::Text(msgtype) = &event.content.msgtype else {
        return;
    };

    let member = room
        .get_member(&event.sender)
        .await
        .expect("Couldn't get the room member")
        .expect("The room member doesn't exist");
    let name = member.name();

    let n = name.to_string();
    let m = msgtype.body.clone();
    slint::invoke_from_event_loop(move || {
        with_ui(|ui| {
            let mut messages: Vec<Message> = ui.get_message_model().iter().collect();

            // Create and add new message
            let msg: Message = Message {
                user: n.into(),
                text: m.into(),
            };
            messages.push(msg);
            ui.set_message_model(slint::VecModel::from_slice(&messages));
        });
    })
    .expect("Failed to update UI");

    //let content = RoomMessageEventContent::text_plain("🎉🎊🥳 let's PARTY!! 🥳🎊🎉");
    //room.send(content).await.unwrap();
    println!("{name}: {}", msgtype.body);
}
