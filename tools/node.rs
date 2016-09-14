extern crate parapet as pp;
extern crate clap;

use pp::{Parapet, Interactive};

const DESCRIPTION: &'static str = "
    If you pass an address, it will connect to an existing node on
    some network, otherwise a new network will be created.
";

fn main() {
    use clap::{App, Arg};

    let matches = App::new("parapet")
        // .version("1.0")
        .author("Dylan <dylanmckay34@gmail.com>")
        .about("Peer-to-peer build system")
        .after_help(DESCRIPTION)
        .arg(Arg::with_name("address")
            .help("The address of an existing node on a network to connect to")
            .index(1))
        .arg(Arg::with_name("interactive")
             .long("interactive")
            .short("i")
            .multiple(true)
            .help("Enables the interactive console"))
        .get_matches();

    let mut parapet = if let Some(address) = matches.value_of("address") {
        println!("connecting to existing network on {}", address);

        Parapet::connect(address).unwrap()
    } else {
        println!("running new network on {}:{}", pp::SERVER_ADDRESS.0, pp::SERVER_ADDRESS.1);

        // Create a new network.
        Parapet::new(pp::SERVER_ADDRESS).unwrap()
    };

    if matches.is_present("interactive") {
        println!("starting interactive console");

        let mut interactive = Interactive(parapet);
        interactive.run().unwrap();
    } else {
        parapet.run().unwrap();
    }
}

