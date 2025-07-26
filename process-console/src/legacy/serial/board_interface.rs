use tokio_serial::SerialPortBuilderExt;

use crate::legacy::errors::TockloaderError;
use crate::legacy::serial::traits::BoardInterface;
use crate::legacy::serial::serial::SerialInterface;

impl BoardInterface for SerialInterface {
    fn open(&mut self) -> Result<(), TockloaderError> {
        // Is it async? It can't be awaited...
        let stream = tokio_serial::new(self.port.clone(), self.baud_rate).open_native_async()?;

        self.stream = Some(stream);

        Ok(())
    }
}
