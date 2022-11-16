use crate::Messaging;
use dioxus::prelude::*;
use dioxus_html::on::MouseEvent;
use futures::StreamExt;
use rfd::FileDialog;
use warp::constellation::file::File;
use warp::raygun::{Message, RayGunAttachment};

// Remember: owned props must implement PartialEq!
#[derive(PartialEq, Eq, Props)]
pub struct Props {
    file: File,
    message: Message,
}

#[allow(non_snake_case)]
pub fn Attachment(cx: Scope<Props>) -> Element {
    let size = format!("{:.3} KB", cx.props.file.size() / 1024);
    let name = cx.props.file.name();

    let rg = cx.consume_context::<Messaging>().unwrap();

    let handle_click = move |evt: MouseEvent| {
        evt.cancel_bubble();

        cx.spawn_forever({
            let rg = rg.to_owned();
            let save_path = FileDialog::new()
                .set_file_name(&cx.props.file.name())
                .set_directory("/")
                .save_file();
            println!("selected save path: {:?}", save_path);

            let conversation_id = cx.props.message.conversation_id();
            let id = cx.props.message.id();
            let name = cx.props.file.name();

            async move {
                match save_path {
                    Some(path) => match rg.download(conversation_id, id, name, path).await {
                        Ok(mut stream) => {
                            while let Some(progress) = stream.next().await {
                                match progress {
                                    warp::constellation::Progression::CurrentProgress {
                                        ..
                                    } => {}
                                    warp::constellation::Progression::ProgressComplete {
                                        name,
                                        total,
                                    } => {
                                        println!(
                                            "{name} downloaded with {} bytes written",
                                            total.unwrap_or_default()
                                        );
                                    }
                                    warp::constellation::Progression::ProgressFailed {
                                        name,
                                        error,
                                        ..
                                    } => {
                                        println!("{name} failed to download with error: {error:?}")
                                    }
                                }
                            }
                        }
                        Err(e) => println!("Error: {:?}", e),
                    },
                    None => {
                        println!("Path not provided");
                    },
                }
            }
        });
    };

    cx.render(rsx! {
        div {
            class: "attachment",
            onclick: handle_click,
            div {
                class: "file-name",
                "{name}"
            },
            div {
                class: "file-size",
                "{size}"
            }
        }
    })
}
