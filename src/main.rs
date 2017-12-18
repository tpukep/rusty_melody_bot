extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;

extern crate rusty_melody_bot;
use rusty_melody_bot::config;
use rusty_melody_bot::storage;

#[derive(StructOpt, Debug)]
#[structopt(name = "rusty_melody_bot", about = "Guess melody telegram bot")]
struct Opt {
    #[structopt(help = "Config file")]
    config: String,
}

fn main() {
    // Read config
    let opt = Opt::from_args();
    let config = config::get(&opt.config).expect("Wrong path for config");

    // Init Database
    let rocks = storage::RocksDB::new(&config.database.path);
    
    let bot = rusty_melody_bot::Bot::new(config.clone(), rocks);
    bot.run();
}
