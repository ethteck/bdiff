use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use notify::Watcher;

pub fn create_watcher<P: Into<PathBuf>>(
    path: P,
    modified: Arc<AtomicBool>,
) -> notify::Result<notify::RecommendedWatcher> {
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if let notify::EventKind::Modify(_) = event.kind {
                    modified.store(true, Ordering::Relaxed);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })?;

    watcher.watch(&path.into(), notify::RecursiveMode::NonRecursive)?;

    Ok(watcher)
}
