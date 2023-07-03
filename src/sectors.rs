use crate::metadata::StorageMeta;
use crate::metadata::SECTOR_CACHE_DIR;
use std::fs::File;
use std::io::Write;
use cid::{Cid, Error};
use std::fs::OpenOptions;
// filecoin
use filecoin_proofs_api::{RegisteredSealProof};

pub struct Epoch(u64);

pub enum SectorStatus {
	Empty,
	Staged,
	Sealed,
	Failed,
}

pub enum DealStatus {
	Unknown,
	Active,
	Expired,
	Slashed,
}

#[derive(Clone, Copy, Debug)]
pub struct DealInfo  {
	data_cid: Cid,
	status: DealStatus,
	owner: Address,
	piece_cid: Cid,
	size: u64,
	price_per_epoch: u64,
	duration: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct SectorInfo {
	sector_id: u64,
	status: SectorStatus,
	comm_d: Cid,
	comm_r: Cid,
	proof: Vec<u8>,
	sector_deals: Vec<DealInfo>,
	sector_pieces: Vec<PieceInfo>,
	ticket: SealTicket,
	seed: SealSeed,
	pre_commit_cid: Cid,
	commit_cid: Cid,
	num_tries: u64,
	rep_update_cid: Cid,

	//Chain stuff
	proof_type: RegisteredSealProof,
	seal_start: Epoch,
	seal_end: Epoch,
	pledge_amount: u64,

	//renewals/expiration
	seal_next_renewal: Epoch,
	seal_renewal_deadline: Epoch,
}

impl Default for SectorInfo {
	fn default() -> SectorInfo {
		SectorInfo {
			sector_id: 0,
			status: SectorStatus::Empty,
			storage_dir: dirs::home_dir().unwrap().join(STORAGE_DIR),
			comm_d: Cid::default(),
			comm_r: Cid::default(),
			proof: Vec::new(),
			sector_deals: Vec::new(),
			sector_pieces: Vec::new(),
			ticket: SealTicket::default(),
			seed: SealSeed::default(),
			pre_commit_cid: Cid::default(),
			commit_cid: Cid::default(),
			num_tries: 0,
			rep_update_cid: Cid::default(),
			proof_type: RegisteredSealProof::StackedDrg2KiBV1_1,
			seal_start: Epoch(0),
			seal_end: Epoch(0),
			pledge_amount: 0,
			seal_next_renewal: Epoch(0),
			seal_renewal_deadline: Epoch(0),
		}
	}
}

pub mod sector {

	use crate::metadata::StorageMeta;
	use cid::multihash::{Code, MultihashDigest};
	use cid::Cid;
	use std::convert::TryFrom;
	use std::fs::File;
	use std::io::Read;
	use filecoin_proofs::{PieceInfo, UnpaddedBytesAmount, generate_piece_commitment, write_and_preprocess, add_piece, compute_comm_d};
	use filecoin_proofs_api::RegisteredSealProof;
	use std::io::BufReader;
	use std::path::PathBuf;
	use std::io;
	use std::io::prelude::*;
	use cid::Error;
	use anyhow::{Result};

	const RAW: u64 = 0x55;

	pub fn generate_cid(data: &[u8]) -> Cid {
		use cid::multihash::{Code, MultihashDigest};
		let hash = Code::Sha2_256.digest(data);
		Cid::new_v1(RAW, hash)
	}

	pub fn read_piece(mut piece: &File) -> Result <PieceInfo>
	{
		let mut buf = vec![0; piece.metadata().unwrap().len() as usize];
		piece.read_to_end(&mut buf)?;
		let unpadded_piece_size = UnpaddedBytesAmount(buf.len() as u64);
		let result = generate_piece_commitment(
		    &*buf,
		    unpadded_piece_size,
		);
		Ok(result.unwrap())
	}

	pub fn generate_comm_d(sector_size: u64, pieces: &[PieceInfo]) -> Result<Cid> {
		let comm_d = compute_comm_d(RegisteredSealProof::StackedDrg2KiBV1, sector_size, pieces);
		Ok(generate_cid(&comm_d.unwrap()))
	}

	//pub fn seal_p

	pub fn write_sector(mut piece_fd: &File, sector_fd: &File) -> Result<(PieceInfo, UnpaddedBytesAmount)> {
		let mut buf = vec![0; piece_fd.metadata().unwrap().len() as usize];
		piece_fd.read_to_end(&mut buf)?;
		let result = write_and_preprocess(&*buf, sector_fd, filecoin_proofs::UnpaddedBytesAmount(piece_fd.metadata().unwrap().len() as u64));
		Ok(result.unwrap())
	}

	pub fn write_sector_aligned(mut piece_fd: &File, sector_fd: &File, piece_alignment: &[UnpaddedBytesAmount]) -> Result<(PieceInfo, UnpaddedBytesAmount)> {
		let mut buf = vec![0; piece_fd.metadata().unwrap().len() as usize];
		piece_fd.read_to_end(&mut buf)?;
		let result = add_piece(&*buf, sector_fd, filecoin_proofs::UnpaddedBytesAmount(piece_fd.metadata().unwrap().len() as u64), &piece_alignment);
		Ok(result.unwrap())
	}

	pub fn start(storage_meta: StorageMeta) -> Result<(), Error> {
		let content_home = storage_meta.storage_dir.join(SECTOR_CONTENT_DIR);
		let sector_cache = storage_meta.storage_dir.join(SECTOR_CACHE_DIR);
		let files = fs::read_dir(home).unwrap();
		for file in files {
			// make piece
			let mut piece_fd = OpenOptions::new().read(true).write(true).create_new(true).open(file);
			let piece = read_piece(&piece_fd);
			let cid = generate_cid(&piece.as_mut().unwrap().commitment);
			let piece_info: PieceInfo = PieceInfo {piece: piece.unwrap(), cid: cid};
			let sector_info = SectorInfo::default();
			sector_info.sector_pieces.push(sector_info);
			let mut sector_fd = OpenOptions::new().read(true).write(true).create_new(true).open(sector_cache.join("sector"))?;
			piece_fd.sync_all();
			sector_fd.sync_all();
			let write_res = write_sector(&piece_fd, &sector_fd).unwrap();
		}
	}
}

// Unused test, needs to be updated 

/*
use super::*;

use std::io::Cursor;

use filecoin_proofs_api::fr32::Fr32Reader;
use filecoin_proofs::{DefaultPieceHasher, CommitmentReader, PieceInfo};
use storage_proofs_core::pieces::generate_piece_commitment_bytes_from_source;
use crate::sectors::sector::read_piece;
use cid::multibase::Base;
use cid::Cid;
use std::fs::OpenOptions;

struct SectorInfo {
	piece: PieceInfo,
	cid: Cid,
}

use filecoin_proofs::types::{PaddedBytesAmount, UnpaddedBytesAmount};
pub fn store_file_test(storage_meta: StorageMeta) -> Result<(), Error> {
	use sector::*;
	use rand::*;
	let home = storage_meta.storage_dir.join(SECTOR_CACHE_DIR);

	println!("Start store_file_test, home {}", home.to_str().unwrap());

	//get rando bytes
	let mut rng = rand::thread_rng();
	let mut rng_buf = [0u8; 2048];
	rng.fill_bytes(&mut rng_buf);

	//make one piece
	let mut piece_fd_1 = OpenOptions::new().read(true).write(true).create_new(true).open(home.join("piece_1"))?;
	piece_fd_1.write_all(&rng_buf[..127])?;
	piece_fd_1.sync_all()?;
	// Generate CID1
	let mut piece_1 = read_piece(&piece_fd_1);
	let cid_1 = generate_cid(&piece_1.as_mut().unwrap().commitment);

	let sector: SectorInfo = SectorInfo {piece: piece_1.unwrap(), cid: cid_1};
	let mut sectors = vec![sector];

	//make another piece
	let mut piece_fd_2 = OpenOptions::new().read(true).write(true).create_new(true).open(home.join("piece_2"))?;
	piece_fd_2.write_all(&rng_buf[..1016])?;
	piece_fd_2.sync_all()?;
	// Generate CID2
	let mut piece_2 = read_piece(&piece_fd_2);
	let cid_2 = generate_cid(&piece_2.as_mut().unwrap().commitment);

	sectors.push(SectorInfo {piece: piece_2.unwrap(), cid: cid_2});

	for (i, sector) in sectors.iter().enumerate() {
		println!("CID {}: {}", i, sector.cid.to_string_of_base(Base::Base64).unwrap());
	}

	//write first piece (no aligment)
	// generate sector file
	let mut sector_fd = OpenOptions::new().read(true).write(true).create_new(true).open(home.join("sector"))?;
	sector_fd.sync_all();
	let write_result_1 = write_sector(&piece_fd_1, &sector_fd).unwrap();
	let total_bytes = [write_result_1.1];

	//write second piece w alignment
	let write_result_2 = write_sector_aligned(&piece_fd_2, &sector_fd, &total_bytes).unwrap();

	assert_eq!((UnpaddedBytesAmount(piece_fd_2.metadata().unwrap().len() as u64) - write_result_1.1), UnpaddedBytesAmount(889));
	assert_eq!(write_result_2.1, UnpaddedBytesAmount(1905));

	let generate_pre_seal = 

	Ok(())
	
/*
    let piece_size = 127 * 8;
    let source = vec![255u8; piece_size];
    let mut fr32_reader = Fr32Reader::new(Cursor::new(&source));

    let commitment1 = generate_piece_commitment_bytes_from_source::<DefaultPieceHasher>(
        &mut fr32_reader,
        PaddedBytesAmount::from(UnpaddedBytesAmount(piece_size as u64)).into(),
    )
    .expect("failed to generate piece commitment bytes from source");

    let fr32_reader = Fr32Reader::new(Cursor::new(&source));
    let mut commitment_reader = CommitmentReader::new(fr32_reader);
    std::io::copy(&mut commitment_reader, &mut std::io::sink()).expect("io copy failed");

    let commitment2 = commitment_reader.finish().expect("failed to finish");

    // Generate CIDA
    let cid_a = parse(source);

    println!("CID A: {}", cid_a.to_string());

    assert_eq!(&commitment1[..], AsRef::<[u8]>::as_ref(&commitment2));
    */
} */