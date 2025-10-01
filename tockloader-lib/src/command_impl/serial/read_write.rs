use async_trait::async_trait;

use crate::bootloader_serial::{issue_command, Command, Response};
use crate::connection::{Connection, SerialConnection};
use crate::errors::{InternalError, TockloaderError};
use crate::ReadWrite;

#[async_trait]
impl ReadWrite for SerialConnection {
    async fn read(&mut self, address: u64, data: &mut [u8]) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let stream = self.stream.as_mut().expect("Board must be open");

        let mut packet = vec![];
        packet.extend_from_slice(&address.to_le_bytes());
        packet.extend_from_slice(&(data.len() as u16).to_le_bytes());
        let (_, read_data) = issue_command(
            stream,
            Command::ReadRange,
            packet,
            true,
            data.len(),
            Response::ReadRange,
        )
        .await?;

        data.copy_from_slice(&read_data);
        Ok(())
    }

    async fn write(&mut self, _address: u64, _data: &[u8]) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }

        let _stream = self.stream.as_mut().expect("Board must be open");
        todo!("Serial write not yet implemented");
    }
}
