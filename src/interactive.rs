use {State, Parapet, Error};

use std::{io, thread};
use std::sync::mpsc::channel;
use std::io::Write;
use std::sync::mpsc::TryRecvError;

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

                let _arguments: Vec<_> = words.collect();

                match command.as_str() {
                    "exit" | "quit" | "q" => break,
                    "help" => tx.send(Message::Command(Command::Help)).unwrap(),
                    "list" => tx.send(Message::Command(Command::List)).unwrap(),
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
                    },
                },
                Err(TryRecvError::Empty) => (), // all good
                Err(TryRecvError::Disconnected) => break,
            }

            self.0.tick()?;
        }

        Ok(())
    }

    pub fn help(&self) {
        println!("<unimplemented>")
    }

    pub fn list(&self) {
        if let State::Connected { ref node, .. } = self.0.state {
            let network = &node.network;

            for node in network.nodes() {
                println!("{} - ({} siblings)", node.uuid, network.siblings(&node.uuid).len());
            }
        } else {
            println!("not yet connected to network");
        }
    }
}

