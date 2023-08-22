use std::path::Path;

use iced::futures::channel::mpsc::{channel, Receiver};
use iced::{futures, subscription, Subscription};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use iced::futures::{SinkExt, StreamExt};

use crate::Message;

pub fn subscription(path: String) -> Subscription<Message> {
    subscription::channel(path.clone(), 10, |mut output| async move {
        let (mut watcher, mut rx) = async_watcher().expect("Failed to create watcher");

        watcher
            .watch(Path::new(&path), RecursiveMode::NonRecursive)
            .unwrap_or_else(|_| panic!("Failed to watch path {:}", path));

        loop {
            while let Some(res) = rx.next().await {
                match res {
                    Ok(event) => {
                        output
                            .send(Message::WatchedFileChanged(event))
                            .await
                            .expect(
                                "Couldn't send a WatchedFileChanged Message for some odd reason",
                            );
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        }
    })
}

pub fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}
