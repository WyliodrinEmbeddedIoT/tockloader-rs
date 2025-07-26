use crate::legacy::serial::serial::SerialInterface;

use crate::legacy::errors::{CLIError, TockloaderError};
use clap::ArgMatches;
use enum_dispatch::enum_dispatch;

use crate::legacy::serial::traits;

#[enum_dispatch(BoardInterface)]
#[enum_dispatch(VirtualTerminal)]
pub enum Interface {
    Serial(SerialInterface),
}

pub fn build_interface(args: &ArgMatches) -> Result<Interface, TockloaderError> {
    Ok(SerialInterface::new(args)?.into())
}
