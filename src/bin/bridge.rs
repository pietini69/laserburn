//! PhotonCAM Bridge & Web Server CLI Daemon
//! Runs on Raspberry Pi, macOS, Linux, and Windows.

use clap::Parser;
use photoncam::server::BridgeServer;

#[derive(Parser, Debug)]
#[command(name = "photoncam-bridge")]
#[command(about = "Headless PhotonCAM Web Remote & TCP Serial Bridge for Raspberry Pi and Laser Workshops", long_about = None)]
struct Args {
    /// HTTP Web UI and REST API port (default: 8080)
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Raw TCP Serial Bridge port for external CAM/CAD connections (default: 2323)
    #[arg(short, long, default_value_t = 2323)]
    tcp_bridge: u16,

    /// Serial port of the laser cutter (optional, e.g. /dev/ttyUSB0 or /dev/ttyACM0)
    #[arg(short, long)]
    device: Option<String>,

    /// Baud rate for serial communication (default: 115200)
    #[arg(short, long, default_value_t = 115200)]
    baud: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let server = BridgeServer::new(args.port, Some(args.tcp_bridge));

    // If initial device specified via CLI, connect immediately
    if let Some(port_name) = args.device {
        if let Ok(mut dm) = server.device.lock() {
            println!("[BRIDGE] Auto-connecting to specified device: {} ({} baud)...", port_name, args.baud);
            let _ = dm.connect(&port_name, args.baud);
        }
    }

    server.run()?;
    Ok(())
}
