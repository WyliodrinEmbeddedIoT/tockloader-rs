use anyhow;
use futures::{SinkExt, StreamExt};
use std::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

/// The ConsoleConnection is represented by a pair of channels
/// a log_reader for receiving data from the board
/// a command_writer for sending commands to the board
#[allow(dead_code)]
pub struct ConsoleConnection<OutItem, InItem> {
    log_reader: Mutex<UnboundedReceiver<OutItem>>,
    command_writer: UnboundedSender<InItem>,
    pub error_reader: Mutex<UnboundedReceiver<String>>,
}

pub async fn init<Codec, InItem>(
    mut stream: SerialStream,
    codec: Codec,
) -> Result<ConsoleConnection<Codec::Item, InItem>, anyhow::Error>
where
    Codec: Decoder + Encoder<InItem> + Send + 'static,
    Codec::Item: Send + 'static,
    InItem: Send + 'static,
    <Codec as Decoder>::Error: Send + std::fmt::Display,
    <Codec as Encoder<InItem>>::Error: Send + std::fmt::Display,
{
    #[cfg(unix)]
    stream.set_exclusive(false)?;

    let (mut stream_writer, mut stream_reader) = Framed::new(stream, codec).split();

    let (log_writer, log_reader) = mpsc::unbounded_channel::<Codec::Item>();
    let (command_writer, mut command_reader) = mpsc::unbounded_channel::<InItem>();
    let (error_writer, error_reader) = mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(cmd) = command_reader.recv() => {
                    if let Err(e) = stream_writer.send(cmd).await {
                        let _ = error_writer.send(e.to_string());
                    }
                }
                some_item = stream_reader.next() => {
                    match some_item {
                        Some(Ok(item)) => {
                            let _ = log_writer.send(item);
                        }
                        Some(Err(e)) => {
                            let _ = error_writer.send(e.to_string());
                        }
                        None => break,
                    }
                }
            }
        }
    });

    Ok(ConsoleConnection {
        log_reader: Mutex::new(log_reader),
        command_writer,
        error_reader: Mutex::new(error_reader),
    })
}

impl<OutItem, InItem> ConsoleConnection<OutItem, InItem> {
    pub async fn raw_read(&self) -> Result<OutItem, anyhow::Error> {
        let mut reader = self.log_reader.lock().unwrap();

        reader
            .recv()
            .await
            .ok_or(anyhow::anyhow!("log_reader has closed"))
    }

    pub async fn raw_write(&self, item: InItem) -> Result<(), anyhow::Error> {
        self.command_writer
            .send(item)
            .map_err(|_| anyhow::anyhow!("command_writer has closed"))
    }
}
