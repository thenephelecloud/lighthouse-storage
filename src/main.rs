/* Prototype main thread. For the purpose of this example solution, 
   the prototype will scan the content directory for new uploads. 
   Once a new file is served, it calculates a local PoST and verifies it, 
   proving it owns the file it was sent. */
use futures::{
   channel::mpsc::{channel, Receiver},
   SinkExt, StreamExt,
};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

use crate::metadata::{StorageMeta, STORAGE_DIR, SECTOR_CONTENT_DIR};
use crate::sectors::sector::prove;

mod metadata;
mod sectors;
mod consensus;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("Initializing directories...");
    let mut home = match dirs::home_dir() {
        Some(home) => home,
        None => return Err("Couldn't init dirs".into()),
    };
    home.push(STORAGE_DIR);
    let storage_meta = match metadata::init_dirs(home) {
        Err(_) => return Err("Couldn't init dirs".into()),
        Ok(storage_meta) => storage_meta,
    };

    println!("Done!");

    println!("Starting sector checks...");
    prove(storage_meta.clone());
/*  Actix content server, will be implemented in the future (needs more performance tweaking)
	Run the go fileserver in the go directory to test the content server
    content_server::start_server(storage_meta);

*/
	/*
    futures::executor::block_on(async {
	    if let res = async_watch(watch_path).await {
	    	match res {
	    		Err(_) => println!("Error watching path!"),
	    		Ok(_) => println!("Watched path changed!"),
	    	}
	    	if res.event.kind == EventKind::Create {
	    		println!("New file detected, starting a sector update...");
	    		start(storage_meta.clone());
	    	}
	    }
	});
    */

	Ok(())
}


fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
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

async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}