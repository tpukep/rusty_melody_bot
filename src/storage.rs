use rocksdb::{DB, Error};
use bincode::{serialize, deserialize, Infinite};

use {Melody, Game};

pub struct RocksDB {
    database: DB
}

impl RocksDB {

    pub fn new(path: &str) -> RocksDB {
        let db = DB::open_default(path).unwrap();
        RocksDB{database: db}
    }

   pub fn store_melody(&self, melody: Melody) -> Result<(), Error> {
        let data: Vec<u8> = serialize(&melody, Infinite).unwrap();
        let key = format!("m{}", melody.id);
        self.database.put(key.as_bytes(), &data)
   }

    pub fn load_melody(&self, melody_id: u64) -> Result<Option<Melody>, Error> {
        let key = format!("m{}", melody_id);

        match self.database.get(key.as_bytes()) {
            Ok(Some(data)) => {
                let melody: Melody = deserialize(&data[..]).unwrap();
                println!("Retrieved value {:?}", melody);
                Ok(Some(melody))

            },
            Ok(None) => Ok(None),
            Err(e) => Err(e)
        }
    }

    pub fn delete_melody(&self, melody_id: u64) -> Result<(), Error> {
        let key = format!("m{}", melody_id);
        self.database.delete(key.as_bytes())
    }

    pub fn store_game(&self, game: Game) -> Result<(), Error> {
        let data: Vec<u8> = serialize(&game, Infinite).unwrap();
        let key = format!("g{}", game.chat_id);
        self.database.put(key.as_bytes(), &data)
    }

    pub fn delete_game(&self, chat_id: i64) -> Result<(), Error> {
        let key = format!("g{}", chat_id);
        self.database.delete(key.as_bytes())
    }

    pub fn load_answer_for_game(&self, chat_id: i64) -> Result<Option<String>, Error> {
        let key = format!("g{}", chat_id);

        match self.database.get(key.as_bytes()) {
            Ok(Some(data)) => {
                let game: Game = deserialize(&data[..]).unwrap();
                println!("Retrieved value {:?}", game);
                Ok(Some(game.right_answer))

            },
            Ok(None) => Ok(None),
            Err(error) => Err(error)
        }
    }
}
