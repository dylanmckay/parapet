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
        .author("Dylan <me@dylanmckay.io>")
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
        .arg(Arg::with_name("local")
            .long("local")
            .short("l")
            .help("Connect to an existing node running locally on the default port"))
        .get_matches();

    let address: Option<String> = if let Some(address) = matches.value_of("address") {
        Some(address.to_string())
    } else if matches.is_present("local") {
        // Connect to a existing local node.
        Some(format!("{}:{}", pp::SERVER_ADDRESS.0, pp::SERVER_ADDRESS.1))
    } else {
        None
    };

    let mut parapet = if let Some(address) = address {
        println!("connecting to existing network on {}", address);

        Parapet::connect(&*address).unwrap()
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

