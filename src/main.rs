#[tokio::main]
async fn main() -> std::process::ExitCode {
    mywardrobehelper::cli::run().await
}
