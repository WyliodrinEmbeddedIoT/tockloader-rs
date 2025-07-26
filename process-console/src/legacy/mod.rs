use clap::ArgMatches;

use crate::legacy::errors::TockloaderError;
use crate::legacy::serial::interface::build_interface;
use crate::legacy::serial::traits::BoardInterface;
use crate::legacy::serial::traits::VirtualTerminal;

mod errors;
mod serial;

pub async fn run(sub_matches: &ArgMatches) -> Result<(), TockloaderError> {
    let mut interface = build_interface(sub_matches)?;
    interface.open()?;
    interface.run_terminal().await?;

    Ok(())
}