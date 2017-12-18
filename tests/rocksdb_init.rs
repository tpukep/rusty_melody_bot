extern crate rusty_melody_bot;

use rusty_melody_bot::storage::RocksDB;
use rusty_melody_bot::Melody;

const FILE_ID: &'static str = "CQADAgADtwADLNGxSQ1rS-qGOlwMAg";
const CHAT_ID: i32 = 240776749;
const DB_PATH: &'static str = "/tmp/melodies_db";

#[test]
fn rocksdb_init() {
    const ID: u64 = 1;

    let db = RocksDB::new(DB_PATH);

	let melody = Melody{
		id: ID,
		file_id: FILE_ID.to_string(),
		right_answer: "Scarborough Fair".to_string(),
		wrong_answers: [String::from("Green sleeves"), String::from("Morrison Jig"), String::from("Pied Piper")]
	};

	db.store_melody(melody).unwrap();

	let result = db.load_melody(ID);

    match result {
    	Ok(None) => println!("value not found"),
        Ok(melody) => println!("Retrieved value {:?}", melody),
        Err(e) => println!("operational problem encountered: {}", e),
    }

    // db.delete_melody(ID).unwrap();
}