use Slave;
use job;

use gaol::profile::{AddressPattern, Operation, OperationSupport, OperationSupportLevel};
use gaol::profile::{PathPattern, Profile};
use gaol::sandbox::{Command, Sandbox, SandboxMethods};
use std::path::PathBuf;

// Create the sandbox profile.
fn profile() -> Profile {
    // Set up the list of desired operations.
    let mut operations = vec![
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from("/lib"))),
        Operation::FileReadAll(PathPattern::Literal(PathBuf::from("/etc"))),
        Operation::NetworkOutbound(AddressPattern::All),
        Operation::SystemInfoRead,
    ];

    // Remove operations not supported by this OS. (Otherwise the creation of the profile will
    // fail.)
    operations.retain(|operation| {
        println!("{:?}: {:?}", operation, operation.support());
        match operation.support() {
            OperationSupportLevel::NeverAllowed | OperationSupportLevel::CanBeAllowed => true,
            _ => false,
        }
    });

    Profile::new(operations).unwrap()
}

pub struct Gaol;

impl Slave for Gaol
{
    fn run(&mut self, command: job::Command) -> job::run::TaskOutput {
        let mut cmd = Command::new(command.executable);
        cmd.args(&command.arguments);
        Sandbox::new(profile()).start(&mut cmd).unwrap().wait().unwrap();
        unimplemented!();
    }
}

