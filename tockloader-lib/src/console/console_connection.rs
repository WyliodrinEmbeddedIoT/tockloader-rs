use anyhow;
use bytes::Bytes;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

/// The ConsoleConnection is represented by a pair of channels
/// a log_reader for receiving data from the board
/// a command_writer for sending commands to the board
#[allow(dead_code)]
pub struct ConsoleConnection {
    log_reader: UnboundedReceiver<Bytes>,
    command_writer: UnboundedSender<Bytes>,
    error_reader: UnboundedReceiver<String>,
}

pub async fn init(tty: &str) -> Result<ConsoleConnection, anyhow::Error> {
    let mut stream: SerialStream = tokio_serial::new(tty, 115200).open_native_async()?;

    #[cfg(unix)]
    stream.set_exclusive(false)?;

    let (mut stream_reader, mut stream_writer) = split(stream);

    let (log_writer, log_reader) = mpsc::unbounded_channel::<Bytes>();
    let (command_writer, mut command_reader) = mpsc::unbounded_channel::<Bytes>();
    let (error_writer, error_reader) = mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];

        loop {
            tokio::select! {
                Some(cmd) = command_reader.recv() => {
                    let _ = stream_writer.write(&cmd).await;
                }
                stream_read_result = stream_reader.read(&mut buffer) => {
                    match stream_read_result {
                        Ok(len) => {
                            let _ = log_writer.send(Bytes::copy_from_slice(&buffer[..len]));
                        }
                        Err(e) => {
                            let _ = error_writer.send(e.to_string());
                        }
                    }
                }
            }
        }
    })
    .await?;

    Ok(ConsoleConnection {
        log_reader,
        command_writer,
        error_reader,
    })
}

impl ConsoleConnection {}
