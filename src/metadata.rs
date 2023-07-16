use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::consensus::eth_spec::EthSpecId;

pub static STORAGE_DIR : &str = "validator_storage";
pub static SECTOR_CACHE_DIR : &str = "sector-cache";
pub static SECTOR_SEALED_DIR : &str = "sector-sealed";
pub static SECTOR_STAGED_DIR : &str = "sector-staged";
pub static SECTOR_CONTENT_DIR : &str = "sector-content";

#[derive(Deserialize, Serialize, Debug)]
pub struct StorageMeta {
	pub storage_size: i64, // Storage size in Mib
	pub sector_size: i64, // Sector size in Mib
	pub storage_dir: PathBuf, // Path we store
	init: bool, // Init flag
}

impl Default for StorageMeta {
	fn default() -> StorageMeta {
		StorageMeta {
			storage_size: 0,
			sector_size: 0,
			storage_dir: dirs::home_dir().unwrap().join(STORAGE_DIR),
			init: true,
		}
	}
}

impl Clone for StorageMeta {
	fn clone(&self) -> StorageMeta {
		StorageMeta {
			storage_size: self.storage_size,
			sector_size: self.sector_size,
			storage_dir: self.storage_dir.clone(),
			init: self.init,
		}
	}
}

impl StorageMeta {
    pub fn mainnet() -> Self {
        Self {
			storage_size: 256 * 1024 * 1024,
			sector_size: 64 * 1024 * 1024,
			storage_dir: dirs::home_dir().unwrap().join(STORAGE_DIR),
			init: true,
        }
    }

    pub fn minimal() -> Self {
        Self {
			storage_size: 1 * 1024 * 1024,
			sector_size: 512,
			storage_dir: dirs::home_dir().unwrap().join(STORAGE_DIR),
			init: true,
        }
    }

    pub fn testnet() -> Self {
        Self {
			storage_size: 1 * 1024 * 1024,
			sector_size: 512,
			storage_dir: dirs::home_dir().unwrap().join(STORAGE_DIR),
			init: true,
        }
    }
    pub fn new(spec: EthSpecId) -> Self {
    	match spec {
			EthSpecId::Mainnet => Self::mainnet(),
			EthSpecId::Minimal => Self::minimal(),
			EthSpecId::Gnosis => Self::testnet(),
		}
    }
}

fn read_config(home: PathBuf) -> Result<StorageMeta, std::io::Error> {
	let storage_conf = match std::fs::read_to_string(home.join("storage_conf.json"))
	{
		Err(why) => panic!("couldn't read storage_conf.json: {}", why),
		Ok(storage_conf) => storage_conf,
	};
	Ok(serde_json::from_str::<StorageMeta>(&storage_conf).unwrap())
}

fn write_config(storage_meta: StorageMeta, home: PathBuf) -> Result<(), std::io::Error> {
	let storage_conf = serde_json::to_string(&storage_meta).unwrap();
	fs::write(home.join("storage_conf.json"), storage_conf).unwrap();
	Ok(())
}

pub fn init_dirs(home: PathBuf) -> Result<StorageMeta, std::io::Error> {

	// TODO argument support

	if !home.exists() {
		//TODO should move to using existing validator_client home directory
		fs::create_dir(dirs::home_dir().unwrap().join(STORAGE_DIR)).expect("Failed to create home directory");
	}

	let mut storage_meta = StorageMeta::default();

	if (home.join("storage_conf.json")).exists() {
		storage_meta = match read_config(home.clone()) {
			Err(_) => panic!("Couldn't read storage_conf.json"),
			Ok(storage_meta) => storage_meta,
		};
	}

	if !storage_meta.storage_dir.join(SECTOR_CACHE_DIR).exists() || storage_meta.storage_dir.join(SECTOR_SEALED_DIR).exists() || storage_meta.storage_dir.join(SECTOR_STAGED_DIR).exists() {
		storage_meta.init = true;
	}

	if storage_meta.init {

		if storage_meta.storage_dir.join(SECTOR_CACHE_DIR).exists() {
			fs::remove_dir_all(storage_meta.storage_dir.join(SECTOR_CACHE_DIR)).expect("Failed to remove sector-cache directory");
		}
		if storage_meta.storage_dir.join(SECTOR_SEALED_DIR).exists() {
			fs::remove_dir_all(storage_meta.storage_dir.join(SECTOR_SEALED_DIR)).expect("Failed to remove sector-sealed directory");
		}
		if storage_meta.storage_dir.join(SECTOR_STAGED_DIR).exists() {
			fs::remove_dir_all(storage_meta.storage_dir.join(SECTOR_STAGED_DIR)).expect("Failed to remove sector-staged directory");
		}
		if storage_meta.storage_dir.join(SECTOR_CONTENT_DIR).exists() {
			fs::remove_dir_all(storage_meta.storage_dir.join(SECTOR_CONTENT_DIR)).expect("Failed to remove sector-content directory");
		}

		fs::create_dir(storage_meta.storage_dir.join(SECTOR_CACHE_DIR)).expect("Failed to create sector-cache directory");

		fs::create_dir(storage_meta.storage_dir.join(SECTOR_SEALED_DIR)).expect("Failed to create sector-sealed directory");

		fs::create_dir(storage_meta.storage_dir.join(SECTOR_STAGED_DIR)).expect("Failed to create sector-staged directory");

		fs::create_dir(storage_meta.storage_dir.join(SECTOR_CONTENT_DIR)).expect("Failed to create sector-content directory");

		storage_meta.init = false;

		match write_config(storage_meta.clone(), home) {
			Err(_) => panic!("Couldn't write to storage_conf.json"),
			Ok(_) => println!("Successfully wrote to storage_conf.json"),
		}
	}

	Ok(storage_meta)
}