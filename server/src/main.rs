use tokio::io;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Binding server!");
    // bind server
    let tcp_listener = TcpListener::bind("localhost:6142").await.unwrap();

    loop { 
        // when new clients register the connection, create a socket and adddress object
        let (mut socket, _address) = tcp_listener.accept().await.unwrap();

        // with the new socket and address, create a new async block to handle its messages.
        // move means to capture the current closure by value.
        tokio::spawn(async move {
            let (mut read, mut write) = socket.split();
            println!("split socket");

            // since data that is enclosed in an await needs to be saved as a kind of closure by the Task, 
            // we would prefer to allocate this buffer on the heap, since anything on the stack will 
            // have to be copied for each await call.
            let mut message_buffer = vec![0; 1025];
            println!("Initialised msg buffer");

            // enter echo loop
            // in each loop, await receiving reads, then immediately send back 
            loop {
                // wait for reaad available and then read
                let bytes_read = read.read(&mut message_buffer).await.unwrap();
                println!("Message Received");
                if bytes_read == 0 {
                    return;
                }
                let message = String::from_utf8(message_buffer.clone()).unwrap();
                println!("{}", message);

                // write the contents of the buffer straight back into the writer.
                write.write_all(&mut message_buffer[..bytes_read]).await.unwrap();
                println!("Writing back");
            }
        });
    }
}
