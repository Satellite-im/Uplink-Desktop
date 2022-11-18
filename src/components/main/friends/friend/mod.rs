use dioxus::prelude::*;
use dioxus_heroicons::outline::Shape;
use ui_kit::{
    activity_indicator::ActivityIndicator,
    icon_button::IconButton,
    profile_picture::PFP,
    skeletons::{inline::InlineSkeleton, pfp::PFPSkeleton},
};
use warp::{crypto::DID, error::Error, raygun::Conversation};

use crate::{
    state::{Actions, ConversationInfo},
    utils_internal, Messaging, STATE,
};
use ::utils::Account;

#[derive(Props)]
pub struct Props<'a> {
    account: Account,
    messaging: Messaging,
    friend: DID,
    on_chat: EventHandler<'a, ()>,
}

#[allow(non_snake_case)]
pub fn Friend<'a>(cx: Scope<'a, Props>) -> Element<'a> {
    log::debug!("rendering Friend");
    let state = use_atom_ref(&cx, STATE);

    let mp = cx.props.account.clone();
    let rg = cx.props.messaging.clone();

    let username = utils_internal::get_username_from_did(cx.props.friend.clone(), &mp);
    let show_skeleton = username.is_empty();

    let profile_picture = utils_internal::get_pfp_from_did(cx.props.friend.clone(), &mp);

    cx.render(rsx! {
        div {
            class: "friend",
            if show_skeleton {rsx!(
                PFPSkeleton {}
            )} else {rsx!(
                rsx!(PFP {
                    src: profile_picture,
                    size: ui_kit::profile_picture::Size::Normal
                })
            )},
            div {
                class: "who",
                if show_skeleton {rsx!(
                    InlineSkeleton {}
                )} else {rsx!(
                    h3 {
                        "{username}"
                    },
                    ActivityIndicator {
                        inline: true,
                        remote_did: cx.props.friend.clone(),
                        account: cx.props.account.clone(),
                    }
                )}
            },
            div {
                class: "request-controls",
                div {
                    class: "control-wrap",
                    if show_skeleton {rsx!(
                        IconButton {
                            icon: Shape::ChatAlt,
                            disabled: true,
                            on_pressed: move |_| {}
                        }
                    )} else {rsx!(
                        IconButton {
                            icon: Shape::ChatAlt,
                            on_pressed: move |_| {
                                let rg = rg.clone();
                                let friend = cx.props.friend.clone();
                                let conversation_response = warp::async_block_in_place_uncheck(
                                    rg.write().create_conversation(&friend)
                                );
                                let conversation = match conversation_response {
                                    Ok(v) => v,
                                    Err(Error::ConversationExist { conversation }) => conversation,
                                    Err(_) => Conversation::default(),
                                };
                                state.write().dispatch(Actions::ChatWith(ConversationInfo{conversation, ..Default::default() }));
                                cx.props.on_chat.call(());
                            }
                        }
                    )}
                }
            }
        }
    })
}
