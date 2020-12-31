use server::Server;

mod connection;
mod error;
mod protocol;
mod server;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn start() {
    if let Err(error) = __start() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

pub fn __start() -> Result<()> {
    let (connection, threads) = connection::stdio();

    Server::new(connection).initialize()?.run()?;

    threads.join()?;
    log::info!("Connection to client has closed");

    Ok(())
}
