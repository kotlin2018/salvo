use salvo::prelude::*;

#[fn_handler]
async fn hello_world() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new().get(hello_world);
    // 同时启动两个服务，分别访问 http://127.0.0.1:7878/ http://127.0.0.1:7979/ 都能调用到 hello_world 这个handler
    let listener = TcpListener::bind("127.0.0.1:7878").join(TcpListener::bind("127.0.0.1:7979"));
    tracing::info!("Listening on http://127.0.0.1:7878");
    tracing::info!("Listening on http://127.0.0.1:7979");
    Server::new(listener).serve(router).await;
}
