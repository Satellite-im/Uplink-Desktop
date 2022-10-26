use crate::{
    components::ui_kit::skeletons::{inline::InlineSkeleton, pfp::PFPSkeleton},
    state::{Actions, ConversationInfo},
    Account, Messaging, LANGUAGE, STATE,
};
use dioxus::prelude::*;
use futures::stream::StreamExt;
use warp::raygun::{MessageEventKind, MessageOptions, RayGun, RayGunStream};

#[derive(Props)]
pub struct Props<'a> {
    account: Account,
    conversation_info: ConversationInfo,
    messaging: Messaging,
    on_pressed: EventHandler<'a, ()>,
}

#[allow(non_snake_case)]
pub fn Chat<'a>(cx: Scope<'a, Props<'a>>) -> Element<'a> {
    let state = use_atom_ref(&cx, STATE);
    let state2 = state.clone();
    let l = use_atom_ref(&cx, LANGUAGE).read();
    let unread_count = use_state(&cx, || 0_usize).clone();
    let is_active = use_state(&cx, || false);

    let mut rg = cx.props.messaging.clone();
    let mp = cx.props.account.clone();

    let ident = mp
        .read()
        .get_own_identity()
        .expect("Unexpected error <temp>");

    let username = cx
        .props
        .conversation_info
        .conversation
        .recipients()
        .iter()
        //filters out our own did key in the iter
        .filter(|did| ident.did_key().ne(did))
        //tries get_identity so if it returns Option::Some it would be the map item, otherwise its filtered out
        .filter_map(|did| mp.read().get_identity(did.clone().into()).ok())
        //flatted the nested iterators
        .flatten()
        .map(|i| i.username())
        .last()
        .unwrap_or_default();

    let show_skeleton = username.is_empty();

    let (active, _is_active) = match state.read().current_chat.as_ref() {
        Some(active) => {
            if *active == cx.props.conversation_info.conversation.id() {
                ("active", true)
            } else {
                ("none", false)
            }
        }
        None => ("", false),
    };

    if is_active != _is_active {
        is_active.set(_is_active);
    }

    let mut conversation_info = cx.props.conversation_info.clone();
    use_future(
        &cx,
        (&unread_count, &is_active.clone()),
        |(unread_count, is_active)| async move {
            let messages = rg
                .get_messages(
                    conversation_info.conversation.id(),
                    MessageOptions::default(),
                )
                .await
                .unwrap_or_default();

            if *is_active {
                if let Some(msg) = messages.last() {
                    // todo: prevent unnecessary writes
                    conversation_info.last_msg_read = Some(msg.id());
                    state2
                        .write_silent()
                        .dispatch(Actions::UpdateConversation(conversation_info.clone()))
                        .save();
                    unread_count.set(0);
                }
                // very important: don't open two message streams - if this is the active chat, the messages Element will read the stream and this
                // chat component shouldn't.
                return;
            } else {
                let num_unread_messages = match conversation_info.last_msg_read {
                    Some(id) => messages.iter().rev().take_while(|x| x.id() != id).count(),
                    None => messages.len(),
                };

                if *unread_count.current() != num_unread_messages {
                    unread_count.set(num_unread_messages);
                }
            }

            let mut stream = loop {
                match rg
                    .get_conversation_stream(conversation_info.conversation.id())
                    .await
                {
                    Ok(stream) => break stream,
                    Err(e) => match &e {
                        warp::error::Error::RayGunExtensionUnavailable => {
                            //Give sometime for everything in the background to fully line up
                            //Note, if this error still happens, it means there is an fatal error
                            //      in the background
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        _ => {
                            // todo: properly report this error
                            // eprintln!("failed to get_conversation_stream: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                        }
                    },
                }
            };

            while let Some(event) = stream.next().await {
                if let MessageEventKind::MessageReceived { message_id, .. } = event {
                    if *is_active {
                        conversation_info.last_msg_read = Some(message_id);
                        state2
                            .write_silent()
                            .dispatch(Actions::UpdateConversation(conversation_info.clone()))
                            .save();
                    } else {
                        unread_count.modify(|x| x + 1);
                    }
                }
            }
        },
    );

    if show_skeleton {
        cx.render(rsx! {
            div {
                class: "chat {active}",
                onclick: move |_| cx.props.on_pressed.call(()),
                PFPSkeleton {},
                div {
                    class: "who",
                    InlineSkeleton {},
                    InlineSkeleton {}
                }
            }
        })
    } else {
        cx.render(rsx! {
            div {
                class: "chat {active}",
                onclick: move |_| {
                    cx.props.on_pressed.call(());
                    is_active.set(true);
                },
                div {
                    class: "pfp"
                },
                div {
                    class: "who",
                    div {
                        class: "top-row",
                        h3 {
                            "{username}"
                        },
                        span {
                            class: "timestamp",
                            "10:00am"
                        }
                    },
                    span {
                        match *unread_count.current() {
                            0 => rsx!("{l.chat_placeholder}"),
                            _ => rsx!("unread: {unread_count}"),
                        }
                    }
                }
            }
        })
    }
}
