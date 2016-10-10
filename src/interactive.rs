use {Parapet, Error, PacketKind, job};
use ci;
use network::local;

use std::{io, thread};
use std::sync::mpsc::channel;
use std::io::Write;
use std::sync::mpsc::TryRecvError;
use protocol;

use uuid::Uuid;

pub struct Interactive(pub Parapet);

#[derive(Clone, Debug)]
pub enum Message
{
    /// We want to prompt for a command.
    Prompt,

    /// No command was entered.
    Empty,

    /// A command was entered.
    Command(Command),
}

#[derive(Clone, Debug)]
pub enum Command
{
    /// An unknown command was entered.
    Unknown(String),

    Help,

    /// List the nodes in the network.
    List,

    /// Run a command over the network.
    Run {
        executable: String,
        arguments: Vec<String>
    },
}

impl Interactive
{
    pub fn run(&mut self) -> Result<(), Error> {
        let (tx, rx) = channel();

        // We can't print to stdout in this thrad, else
        // it has race conditions with the other thread.
        thread::spawn(move|| {
            loop {
                tx.send(Message::Prompt).unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                let mut words = input.split_whitespace();

                let command = if let Some(cmd) = words.next() {
                    cmd.to_lowercase()
                } else {
                    tx.send(Message::Empty).unwrap();
                    continue;
                };

                let arguments: Vec<_> = words.collect();

                match command.as_str() {
                    "exit" | "quit" | "q" => break,
                    "help" => tx.send(Message::Command(Command::Help)).unwrap(),
                    "list" => tx.send(Message::Command(Command::List)).unwrap(),
                    "run" => {

                        tx.send(Message::Command(Command::Run {
                            executable: arguments[0].to_owned(),
                            arguments: arguments[1..].iter().map(|s| s.to_string()).collect(),
                        })).unwrap();
                    },
                    _ => {
                        tx.send(Message::Command(Command::Unknown(command))).unwrap();
                        continue;
                    },
                }
            }

            drop(tx);
        });

        loop {
            match rx.try_recv() {
                Ok(command) => match command {
                    Message::Prompt => {
                        print!("> ");
                        io::stdout().flush().unwrap();
                    },
                    Message::Empty => println!("no command given"),
                    Message::Command(cmd) => match cmd {
                        Command::Help => self.help(),
                        Command::List => self.list(),
                        Command::Unknown(cmd) => println!("unknown command '{}'", cmd),
                        Command::Run { executable, arguments }=> self.run_command(&executable, &arguments),
                    },
                },
                Err(TryRecvError::Empty) => (), // all good
                Err(TryRecvError::Disconnected) => break,
            }

            match self.0.tick() {
                Ok(..) => (),
                Err(Error::Stop { reason }) => {
                    println!("stopping: {}", reason);
                    break;
                },
                e => return e,
            }
        }

        Ok(())
    }

    pub fn help(&self) {
        println!("<unimplemented>")
    }

    pub fn list(&self) {
        if let local::Node::Connected { ref node, .. } = self.0.node {
            let network = &node.network;

            for network_node in network.nodes() {
                print!("{} - ({} siblings)", network_node.uuid, network.siblings(&network_node.uuid).len());
                if network_node.uuid == node.uuid {
                    println!(" (current)");
                } else {
                    println!("");
                }
            }
        } else {
            println!("not yet connected to network");
        }
    }

    pub fn run_command(&mut self, executable: &str, arguments: &[String]) {
        if let local::Node::Connected { ref mut node, .. } = self.0.node {
            let work = ci::build::Work {
                uuid: Uuid::new_v4(),
                tasks: vec![job::Task {
                    uuid: Uuid::new_v4(),
                    command: job::Command {
                        executable: executable.to_owned(),
                        arguments: arguments.to_owned(),
                    },
                }].into_iter().collect(),
            };

            node.broadcast_packet(&PacketKind::WorkRequest(protocol::WorkRequest::from_work(&work))).unwrap();
        } else {
            println!("not yet connected to network");
        }
    }
}

