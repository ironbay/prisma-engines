pub fn run_with_tokio<O, F: std::future::Future<Output = O>>(fut: F) -> O {
    tokio_runtime().block_on(fut)
}

pub fn tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
}
