use async_trait::async_trait;
use probe_rs::MemoryInterface;

use crate::connection::{Connection, ProbeRSConnection};
use crate::errors::{InternalError, TockloaderError};
use crate::ReadWrite;

#[async_trait]
impl ReadWrite for ProbeRSConnection {
    async fn read(&mut self, address: u64, data: &mut [u8]) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");

        let mut core = session.core(self.target_info.core)?;

        core.read(address, data)?;

        Ok(())
    }

    async fn write(&mut self, address: u64, data: &[u8]) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");

        let mut core = session.core(self.target_info.core)?;

        core.write(address, data)?;

        Ok(())
    }
}
