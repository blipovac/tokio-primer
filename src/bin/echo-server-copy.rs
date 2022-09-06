use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    // ? operator unwraps the value or propagates the error further to the function caller
    // can be used on both `Result<>` and `Option<>`
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    // use split when 
    let (mut reader_handle, mut writer_handle) = io::split(socket);

    let (mut socket, _) = listener.accept().await?;

    tokio::spawn(async move {
        writer_handle.write_all(b"hello\r\n").await?;
        writer_handle.write_all(b"world\r\n").await?;

        Ok::<_, io::Error>(())
    });

    let mut buffer = vec![0; 128];

    loop {
        let n = reader_handle.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        println!("GOT {:?}", &buffer[..n]);
    }

    Ok(())
}
