use crate::{
    main::{compose::Compose, sidebar::Sidebar},
    state::{Actions, ConversationInfo},
    Account, Messaging, STATE,
};
use dioxus::prelude::*;
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;
use warp::raygun::RayGun;

pub mod compose;
pub mod files;
pub mod friends;
pub mod profile;
pub mod settings;
pub mod sidebar;

#[derive(Props, PartialEq)]
pub struct Prop {
    account: Account,
    messaging: Messaging,
}

#[allow(non_snake_case)]
pub fn Main(cx: Scope<Prop>) -> Element {
    let st = use_atom_ref(&cx, STATE).clone();
    let rg = cx.props.messaging.clone();

    use_future(&cx, (), |_| async move {
        loop {
            if let Ok(list) = rg.list_conversations().await {
                let mut current_conversations: HashMap<Uuid, ConversationInfo> = HashMap::new();
                for item in &list {
                    let to_insert = match st.read().all_chats.get(&item.id()) {
                        Some(v) => v.clone(),
                        None => ConversationInfo {
                            conversation: item.clone(),
                            ..Default::default()
                        },
                    };
                    current_conversations.insert(item.id(), to_insert);
                }
                if current_conversations != st.read().all_chats {
                    st.write()
                        .dispatch(Actions::AddRemoveConversations(current_conversations));
                }
            }
            // TODO: find a way to sync this with the frame rate or create a "polling rate" value we can configure
            // This also doesn't really seem to effect performance
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    cx.render(rsx! {
        div {
            class: "main",
            Sidebar {
                messaging: cx.props.messaging.clone(),
                account: cx.props.account.clone(),
            },
            Compose {
                account: cx.props.account.clone(),
                messaging: cx.props.messaging.clone(),
            },
        }
    })
}
