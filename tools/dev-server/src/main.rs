use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::json;
use tracing::{debug, info, warn};
use wtransport::endpoint::IncomingSession;
use wtransport::{Endpoint, Identity, ServerConfig};

#[derive(Debug)]
struct Args {
    listen: SocketAddr,
    cert: Option<PathBuf>,
    key: Option<PathBuf>,
    reject: bool,
    accept_delay: Duration,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut listen = "127.0.0.1:4433".parse()?;
        let mut cert = None;
        let mut key = None;
        let mut reject = false;
        let mut accept_delay = Duration::ZERO;
        let mut args = std::env::args().skip(1);
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--listen" => {
                    listen = args
                        .next()
                        .context("--listen requires an address")?
                        .parse()?;
                }
                "--cert" => cert = Some(args.next().context("--cert requires a path")?.into()),
                "--key" => key = Some(args.next().context("--key requires a path")?.into()),
                "--reject" => reject = true,
                "--accept-delay-ms" => {
                    accept_delay = Duration::from_millis(
                        args.next()
                            .context("--accept-delay-ms requires a value")?
                            .parse()?,
                    );
                }
                "--help" | "-h" => {
                    println!(
                        "Usage: godot-wtransport-dev-server [--listen IP:PORT] [--cert FILE --key FILE] [--reject] [--accept-delay-ms N]"
                    );
                    std::process::exit(0);
                }
                _ => bail!("unknown argument: {argument}"),
            }
        }
        if cert.is_some() != key.is_some() {
            bail!("--cert and --key must be specified together");
        }
        Ok(Self {
            listen,
            cert,
            key,
            reject,
            accept_delay,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "godot_wtransport_dev_server=info".into()),
        )
        .with_writer(std::io::stderr)
        .init();
    let args = Args::parse()?;
    let identity = match (&args.cert, &args.key) {
        (Some(cert), Some(key)) => Identity::load_pemfiles(cert, key).await?,
        _ => Identity::self_signed(["localhost", "127.0.0.1", "::1"])?,
    };
    let certificate_hash = identity.certificate_chain().as_slice()[0].hash();
    let config = ServerConfig::builder()
        .with_bind_address(args.listen)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(3)))
        .build();
    let server = Endpoint::server(config)?;
    let local_address = server.local_addr()?;
    println!(
        "{}",
        json!({
            "event": "ready",
            "url": format!("https://localhost:{}/echo", local_address.port()),
            "address": local_address.to_string(),
            "certificate_hash": certificate_hash.to_string(),
        })
    );

    loop {
        tokio::select! {
            incoming = server.accept() => {
                let reject = args.reject;
                let delay = args.accept_delay;
                tokio::spawn(async move {
                    if let Err(error) = handle_session(incoming, reject, delay).await {
                        warn!(%error, "session ended");
                    }
                });
            }
            signal = tokio::signal::ctrl_c() => {
                signal?;
                info!("shutdown requested");
                break;
            }
        }
    }
    Ok(())
}

async fn handle_session(incoming: IncomingSession, reject: bool, delay: Duration) -> Result<()> {
    let request = incoming.await?;
    info!(
        authority = request.authority(),
        path = request.path(),
        "session requested"
    );
    if !delay.is_zero() {
        tokio::time::sleep(delay).await;
    }
    if reject {
        request.forbidden().await;
        return Ok(());
    }
    let connection = request.accept().await?;
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        tokio::select! {
            datagram = connection.receive_datagram() => {
                let datagram = datagram?;
                debug!(size = datagram.len(), "datagram received");
                connection.send_datagram(&*datagram)?;
            }
            stream = connection.accept_bi() => {
                let (mut send, mut recv) = stream?;
                tokio::spawn(async move {
                    loop {
                        match recv.read(&mut buffer).await {
                            Ok(Some(size)) => {
                                if send.write_all(&buffer[..size]).await.is_err() { break; }
                            }
                            Ok(None) => { let _ = send.finish().await; break; }
                            Err(_) => break,
                        }
                    }
                });
                buffer = vec![0_u8; 64 * 1024];
            }
            stream = connection.accept_uni() => {
                let mut recv = stream?;
                let connection = connection.clone();
                tokio::spawn(async move {
                    let mut send = match connection.open_uni().await {
                        Ok(opening) => match opening.await { Ok(send) => send, Err(_) => return },
                        Err(_) => return,
                    };
                    let mut local = vec![0_u8; 64 * 1024];
                    loop {
                        match recv.read(&mut local).await {
                            Ok(Some(size)) => { if send.write_all(&local[..size]).await.is_err() { break; } }
                            Ok(None) => { let _ = send.finish().await; break; }
                            Err(_) => break,
                        }
                    }
                });
            }
        }
    }
}
