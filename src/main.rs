mod codec;
mod frame;

use codec::TelnetCodec;
use frame::{Action, TelnetFrame, TelnetOption, TelnetSubnegotiation, TerminalTypeOption};
use futures::sink::SinkExt;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    TelnetServer::listen("127.0.0.1:2000").await?;

    println!("Listening. Press any key to exit...");

    Ok(())
}

struct TelnetServer {}

impl TelnetServer {
    async fn listen(addr: &str) -> Result<TelnetServer, Box<dyn Error>> {
        // Start a TCP listener
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (socket, _) = listener.accept().await?;
            tokio::spawn(Self::handle_connection(socket));
        }
    }

    async fn handle_connection(socket: TcpStream) {
        println!("Connection established: {:?}", socket);

        let mut stream = Framed::new(socket, TelnetCodec::new());

        stream
            .send(TelnetFrame::Command {
                action: Action::Will,
                option: TelnetOption::SuppressGoAhead,
            })
            .await
            .unwrap();

        stream
            .send(TelnetFrame::Command {
                action: Action::Will,
                option: TelnetOption::Echo,
            })
            .await
            .unwrap();

        stream
            .send(TelnetFrame::Command {
                action: Action::Do,
                option: TelnetOption::NegotiateAboutWindowSize,
            })
            .await
            .unwrap();

        stream
            .send(TelnetFrame::Data(b"Hello\r\n".to_vec()))
            .await
            .unwrap();

        stream
            .send(TelnetFrame::Command {
                action: Action::Do,
                option: TelnetOption::TerminalType,
            })
            .await
            .unwrap();

        stream
            .send(
                TelnetFrame::Subnegotiation(TelnetSubnegotiation::TerminalType(
                    TerminalTypeOption::Send,
                ))
                .into(),
            )
            .await
            .unwrap();

        loop {
            match stream.next().await {
                Some(Ok(frame)) => match frame {
                    TelnetFrame::Command { action, option } => {
                        println!("Received command: {:?} {:?}", action, option);
                    }
                    TelnetFrame::Data(data) => {
                        println!("Received data: {:?}", data);
                        stream.send(TelnetFrame::Data(data)).await.unwrap();
                    }
                    TelnetFrame::Subnegotiation(subnegotiation) => {
                        println!("Received subnegotiation: {:?}", subnegotiation);
                    }
                },

                Some(Err(e)) => {
                    println!("Error: {:?}", e);
                    break;
                }

                None => {
                    println!("Connection closed");
                    break;
                }
            }
        }
    }
}
