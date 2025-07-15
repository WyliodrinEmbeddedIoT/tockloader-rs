use anyhow::Error;
use bytes::{Buf, BufMut, BytesMut};
use console::Term;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::io::{self, Write};
use tokio::signal;
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

#[derive(Debug)]
struct TerminalCodec;

pub async fn run(stream: SerialStream) {
    println!("Connecting to board... (press Ctrl+C to stop)");

    let (writer, reader) = Framed::new(stream, TerminalCodec).split();
    let reader_handle = tokio::spawn(listen_serial(reader));
    let writer_handle = tokio::spawn(write_serial(writer));
    tokio::select! {
        _ = reader_handle => {}
        _ = writer_handle => {}
    }
}

async fn listen_serial(
    mut reader: SplitStream<Framed<SerialStream, TerminalCodec>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    let read_task = async {
        while let Some(line) = reader.next().await {
            print!("{}", line.unwrap());
            io::stdout().flush().unwrap();
        }

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    tokio::select! {
        _ = ctrl_c => {
        }
        res = read_task => {
            res?;
        }
    }

    Ok(())
}

async fn write_serial(
    mut writer: SplitSink<Framed<SerialStream, TerminalCodec>, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    let write_task = async {
        loop {
            if let Some(buffer) = get_key().await? {
                writer.send(buffer).await?;
            } else {
                break Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
            }
        }
    };

    tokio::select! {
        _ = ctrl_c => {}
        res = write_task => {
            res?;
        }
    }

    Ok(())
}

async fn get_key() -> Result<Option<String>, Error> {
    let console_result = tokio::task::spawn_blocking(move || Term::stdout().read_key()).await?;

    let key = console_result?;

    Ok(match key {
        console::Key::Unknown => None,
        console::Key::UnknownEscSeq(_) => None,
        console::Key::ArrowLeft => Some("\u{1B}[D".into()),
        console::Key::ArrowRight => Some("\u{1B}[C".into()),
        console::Key::ArrowUp => Some("\u{1B}[A".into()),
        console::Key::ArrowDown => Some("\u{1B}[B".into()),
        console::Key::Enter => Some("\n".into()),
        console::Key::Escape => None,
        console::Key::Backspace => Some("\x7f".into()),
        console::Key::Home => Some("\u{1B}[H".into()),
        console::Key::End => Some("\u{1B}[F".into()),
        console::Key::Tab => Some("\t".into()),
        console::Key::BackTab => Some("\t".into()),
        console::Key::Alt => None,
        console::Key::Del => Some("\u{1B}[3~".into()),
        console::Key::Shift => None,
        console::Key::Insert => None,
        console::Key::PageUp => None,
        console::Key::PageDown => None,
        console::Key::Char(c) => Some(c.into()),
        _ => todo!(),
    })
}

impl Decoder for TerminalCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if source.is_empty() {
            return Ok(None);
        }

        // There may be incomplete utf-8 sequences, so interpret as much as we can.
        // We aren't expecting to get non-utf8 bytes. Otherwise, the decoder would get stuck!
        match str::from_utf8(source) {
            Ok(result_str) => {
                // Release immutable reference to source
                let result = result_str.to_string();

                source.clear();
                Ok(Some(result))
            }
            Err(error) => {
                let index = error.valid_up_to();

                if index == 0 {
                    // Q: Returning Some("") makes it so no other bytes are read in. I have no idea why.
                    // If you find a reason why, please edit this comment.
                    // A: By looking at the documentaion of the 'decode' method, Ok(None) signals
                    // that we need to read more bytes. Otherwise, returning Some("") would call
                    // 'decode' again until Ok(None) is returned.
                    return Ok(None);
                }

                let result = str::from_utf8(&source[..index])
                    .expect("UTF-8 string failed after verifying with 'valid_up_to()'")
                    .to_string();
                source.advance(index);

                Ok(Some(result))
            }
        }
    }
}

impl Encoder<String> for TerminalCodec {
    type Error = Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put(item.as_bytes());
        Ok(())
    }
}
