use tokio::net::TcpListener;

use my_redis::server::Server;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    let server: Server = Default::default();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let server = server.clone();

        tokio::spawn(async move {
            server.process(socket).await;
        });
    }
}

