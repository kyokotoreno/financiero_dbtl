use mysql_common::constants::{CapabilityFlags, StatusFlags};
use mysql_common::packets::HandshakePacket;
use mysql_common::proto::MySerialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::borrow::Cow;
use std::{env, vec};
use std::error::Error;
use std::ffi::{CStr, CString};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3306".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        tokio::spawn(async move {
            let hp = HandshakePacket::new(
                0,
                Cow::from(vec![0u8]),
                0,
                [0; 8],
                Option::None::<Cow<'_, [u8]>>,
                CapabilityFlags::empty(),
                0,
                StatusFlags::empty(),
                Option::None::<Cow<'_, [u8]>>
            );

            let mut buf = vec![0; size_of::<HandshakePacket>()];

            hp.serialize(&mut buf);

            let n = socket.write(&buf).await.expect("failed to write data to socket");

            return;

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
