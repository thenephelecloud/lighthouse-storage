use storage::StorageMeta;

pub mod watchdog {

	pub fn watcher(storage_meta: StorageMeta) {
		let watch_path = storage_meta.storage_dir.join(SECTOR_CONTENT_DIR);
	    futures::executor::block_on(async {
	        if let Err(e) = async_watch(watch_path).await {
	            println!("error: {:?}", e)
	        }
	    });
	}
}