use std::net::Ipv4Addr;

use clap::{ArgEnum, Parser, Subcommand};
use rpassword::read_password_from_tty;

use rbroadlink::{
    Device,

    network::WirelessConnection,
};

/// Command line arguments for the CLI
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Blasts an IR / RF code to the world.
    Blast {
        /// Local IP of this machine. Use this if the broadlink device is on a different subnet.
        #[clap(long, short)]
        local_ip: Option<Ipv4Addr>,

        /// The IP address of the broadlink device.
        device_ip: Ipv4Addr,

        /// The code to send, in hex (e.g. abcdef0123456789)
        code: String,
    },

    /// Connect a broadlink device to the network. Only tested on the RM3 Mini and the RM4 Pro
    Connect {
        /// Prompt for the password interactively
        #[clap(long, short)]
        prompt: bool,

        /// Wireless security mode
        #[clap(arg_enum)]
        security_mode: WirelessConnectionArg,

        /// The name of the wireless network
        ssid: String,

        /// The password of the wireless network
        password: Option<String>,
    },

    /// Learn a code from a broadlink device on the network
    Learn {
        /// Local IP of this machine. Use this if the broadlink device is on a different subnet.
        #[clap(long, short)]
        local_ip: Option<Ipv4Addr>,

        /// The IP address of the broadlink device.
        device_ip: Ipv4Addr,

        /// The type of code to learn
        #[clap(arg_enum)]
        code_type: LearnCodeType,
    },

    /// Lists available broadlink devices on the network
    List {
        /// Local IP of this machine. Use this if the broadlink device is on a different subnet.
        #[clap(long, short)]
        local_ip: Option<Ipv4Addr>,
    },

    /// Get information about a broadlink device
    Info {
        /// Local IP of this machine. Use this if the broadlink device is on a different subnet.
        #[clap(long, short)]
        local_ip: Option<Ipv4Addr>,

        /// The IP address of the broadlink device
        device_ip: Ipv4Addr,
    },
}

#[derive(ArgEnum, Clone, Debug)]
enum LearnCodeType {
    IR,
    RF,
}

#[derive(ArgEnum, Clone, Debug)]
enum WirelessConnectionArg {
    None,
    WEP,
    WPA1,
    WPA2,
    WPA,
}

fn main() -> Result<(), String>{
    // Get the args
    let args = Args::parse();

    // Run the command
    return match args.command {
        Commands::Blast { local_ip, device_ip, code } => blast(local_ip, device_ip, code),
        Commands::Connect { security_mode, ssid, password, prompt } => connect(security_mode, ssid, password, prompt),
        Commands::Learn { local_ip, device_ip, code_type } => learn(local_ip, device_ip, code_type),
        Commands::List { local_ip } => list(local_ip),
        Commands::Info { local_ip, device_ip } => info(local_ip, device_ip),
    }
}

fn blast(local_ip: Option<Ipv4Addr>, device_ip: Ipv4Addr, code: String) -> Result<(), String> {
    // Construct a device directly
    let device = Device::from_ip(device_ip, local_ip).expect("Could not connect to device!");
    let hex_code = hex::decode(code).expect("Invalid code!");

    // Ensure that the device is a remote
    let remote = match device {
        Device::Remote { remote } => remote,
        _ => return Err("Device specified is not a remote!".into()),
    };

    println!("Blasting IR/RF code: {:02X?}", hex_code);
    return remote.send_code(&hex_code);
}

fn connect(sec_mode: WirelessConnectionArg, ssid: String, password: Option<String>, prompt: bool) -> Result<(), String> {
    // Enforce unwrapping the password if using a security mode that requires it.
    let password_prompt = Some("Wireless Password (will not show): ");
    let unwrapped_pass = match sec_mode {
        WirelessConnectionArg::None => "".into(),
        _ => if prompt {
                read_password_from_tty(password_prompt).expect("Could not read password!")
            } else {
                password.expect("This mode requires a password!")
            },
    };

    // Construct the connection information
    let connection = match sec_mode {
        WirelessConnectionArg::None => WirelessConnection::None(&ssid),
        WirelessConnectionArg::WEP => WirelessConnection::WEP(&ssid, &unwrapped_pass),
        WirelessConnectionArg::WPA1 => WirelessConnection::WPA1(&ssid, &unwrapped_pass),
        WirelessConnectionArg::WPA2 => WirelessConnection::WPA2(&ssid, &unwrapped_pass),
        WirelessConnectionArg::WPA => WirelessConnection::WPA(&ssid, &unwrapped_pass),
    };

    // Attempt to have the device connect
    Device::connect_to_network(&connection)
        .expect("Could not connect device to network!");

    println!("Sending connection message with the following information: {:?}", connection);

    return Ok(());
}

fn learn(local_ip: Option<Ipv4Addr>, device_ip: Ipv4Addr, code_type: LearnCodeType) -> Result<(), String> {
    println!("Attempting to learn a code of type {:?}...", code_type);

    // Ensure that the device is a remote
    let device = Device::from_ip(device_ip, local_ip).expect("Could not connect to device!");
    let remote = match device {
        Device::Remote { remote } => remote,
        _ => return Err("Device specified is not a remote!".into()),
    };

    // Try to learn the code
    let code = match code_type {
        LearnCodeType::IR => remote.learn_ir(),
        LearnCodeType::RF => remote.learn_rf(),
    }.expect("Could not learn code from device!");

    let hex_string = hex::encode(&code);
    println!("Got code => {}", hex_string);

    return Ok(());
}

fn list(local_ip: Option<Ipv4Addr>) -> Result<(), String> {
    println!("Searching for devices...");

    // Get the devices
    let devs = Device::list(local_ip).expect("Could not list devices!");

    if devs.len() == 0 {
        println!("No devices found.")
    } else {
        println!("Devices:");

        for dev in devs {
            println!("  {}", dev);
        }
    }

    return Ok(());
}

fn info(local_ip: Option<Ipv4Addr>, device_ip: Ipv4Addr) -> Result<(), String> {
    println!("Getting information for device at {}", device_ip);

    // Construct a device directly
    let device = Device::from_ip(device_ip, local_ip).expect("Could not connect to device!");
    println!("  {}", device);

    return Ok(());
}
