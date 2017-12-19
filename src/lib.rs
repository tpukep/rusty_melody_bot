extern crate core;

extern crate telegram_bot_client;
use telegram_bot_client::{BotFactory, Update};
use telegram_bot_client::Bot as Client;

use tokio_core::reactor;
use futures::{Future, Stream};
use futures::future;

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

extern crate tokio_process;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

extern crate telegram_bot_types;
use telegram_bot_types::request as request;
use telegram_bot_types::response as response;

#[macro_use]
extern crate error_chain;

extern crate rocksdb;

extern crate bincode;

extern crate rand;
use rand::{thread_rng, Rng};

pub mod config;
mod errors;
pub mod storage;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Melody {
    pub id: u64,
    pub file_id: String,
    pub right_answer: String,
    pub wrong_answers: [String; 3]
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Game {
    pub chat_id: i64,
    pub right_answer: String
}

pub struct Bot {
    config: config::AppConfig,
    storage: storage::RocksDB
    // client: telegram_bot_client::Bot,
}

impl Bot {

    pub fn new(config: config::AppConfig, storage: storage::RocksDB) -> Bot {
        Bot{config, storage}
    }

    pub fn run(&self) {
        // Init reactor
        let mut event_loop = reactor::Core::new().unwrap();
        let handle = event_loop.handle();

        // Init listening bot updates
        let factory = BotFactory::new(handle.clone());
        let (client, updates) = factory.new_bot(&self.config.telegram.token);

        // &config, bot.clone(), database.clone(),
        let work = updates
            .for_each(|update| {
                println!("{:?}", update);
                
                match update {
                    Update::Message(msg) => {
                        let task = self.handle_message(client.clone(), msg).then(|_| Ok(()));
                        handle.spawn(task);
                    }
                    
                    _ => {
                        println!("Not matched {:?}", update);
                    }
                };

                Ok(())
            });

        event_loop.run(work).unwrap();
    }

    pub fn handle_message(&self, client: Client, msg: serde_json::Value) -> Box<Future<Item = serde_json::Value, Error = telegram_bot_client::errors::Error>> {
        println!("Handling message: {:?}", msg);

        let msg: response::Message = serde_json::from_value(msg).expect("Unexpected message format");
        let text = msg.text.clone().unwrap_or(String::new());

        match text.chars().nth(0) {
            Some('/') => {
                let mut tokens = text.split_whitespace();
                let cmd = tokens.next().unwrap();
                let args = tokens.next();
                
                self.handle_command(client, msg.chat.id, cmd, args)                
            },

            _ => {
                self.check_answer(client, msg.chat.id, text)
            }
        }
    }

    fn handle_command(&self, client: Client, chat_id: i64, cmd: &str, args: Option<&str>) -> Box<Future<Item = serde_json::Value, Error = telegram_bot_client::errors::Error>> {
        println!("Handling command: {:?} Args: {:?}", cmd, args);

        match (cmd, args) {
            ("/start", None) => {
                println!("Starting chat");
                Box::new(future::ok(serde_json::Value::Null))
            },
            ("/game", None) => {
                println!("Starting game");
                self.start_game(client, chat_id)
            },
            _ => {
                println!("Not supported command");
                Box::new(future::ok(serde_json::Value::Null))
            }
        }
    }

    fn check_answer(&self, client: Client, chat_id: i64, answer: String) -> Box<Future<Item = serde_json::Value, Error = telegram_bot_client::errors::Error>> { 
        println!("Received answer: {:?}", answer);
        match self.storage.load_answer_for_game(chat_id) {
            Ok(Some(right_answer)) => {
                    self.storage.delete_game(chat_id).unwrap();

                    let message = 
                        if answer == right_answer {
                            request::Message::with_keyboard_remover(chat_id, "You win!".to_string());
                        } else {
                            request::Message::with_keyboard_remover(chat_id, "You win!".to_string());
                        };

                    client.request::<_, serde_json::Value>("sendMessage", &message)
                },

            Ok(None) => {
                println!("Game for chat {} is not started", chat_id);

                let message = request::Message::with_keyboard_remover(chat_id, "To start game enter /game command".to_string());
                client.request::<_, serde_json::Value>("sendMessage", &message)
            },

            Err(error) => {
                println!("Failed to load game for chat {}. Error: {:?}", chat_id, error);
                Box::new(future::ok(serde_json::Value::Null))
            }
        }
    }

    fn start_game(&self, client: Client, chat_id: i64) -> Box<Future<Item = serde_json::Value, Error = telegram_bot_client::errors::Error>> {
        // TODO: select random melody
        // let count = self.storage.get_melodies_count();
        // let n = rand.intn(count);

        let id = 1;
        let melody = self.storage.load_melody(id).unwrap().unwrap();
        println!("Loaded: {:?}", melody);

        let game = Game{chat_id, right_answer: melody.right_answer.clone()};
        println!("Storing: {:?}", game);
        self.storage.store_game(game).unwrap();

        let keyboard = make_keyboard_markup(melody.right_answer, melody.wrong_answers);

        let message = request::AudioMessage{
            chat_id,
            audio: melody.file_id,
            reply_markup: Some(keyboard)
        };

        println!("Sending: {:?}", message);

        client.request::<_, serde_json::Value>("sendAudio", &message)
    }
}

fn make_keyboard_markup(right_answer: String, wrong_answers: [String; 3]) -> request::ReplyMarkup {
    let mut answers = Vec::new();
    answers.extend(wrong_answers.iter().cloned());
    answers.push(right_answer);
    
    let mut rng = thread_rng();
    rng.shuffle(&mut answers);

    let mut buttons = Vec::with_capacity(answers.len());

    for answer in answers.into_iter() {
        let button = request::KeyboardButton::new(answer);
        buttons.push(button);
    }

    return request::ReplyMarkup::keyboard_markup(buttons);
}

#[test]
fn test_keyboard_serialize() {
    const BUTTONS_COUNT: usize = 4;

    let mut buttons = Vec::with_capacity(BUTTONS_COUNT);

    for i in 0..BUTTONS_COUNT {
        buttons.push(request::KeyboardButton::new(i.to_string()));
    }

    let keyboard = request::ReplyKeyboardMarkup::new(buttons);

    let data = serde_json::to_string(&keyboard).unwrap();

    println!("{:?}", data);

    let audio = request::Audio::new("1237x1bx12sa".to_string(), 32);

    let audio_data = serde_json::to_string(&audio).unwrap();

    println!("{:?}", audio_data);
}
