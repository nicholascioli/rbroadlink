use std::{collections::HashMap, net::Ipv4Addr, time::Duration};

use clap::Parser;
use log::{info, warn};
use mqtt_async_client::{
    client::{Client, KeepAlive, Publish, QoS, Subscribe, SubscribeTopic},
    Error,
};
use rbroadlink::{traits::DeviceTrait, Device};

#[derive(Parser, Clone, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Add a client to track. Can be repeated.
    #[clap(short, long, multiple_occurrences(true))]
    client: Vec<Ipv4Addr>,

    /// Enable automatically discovering devices
    #[clap(short, long)]
    auto_discover: bool,

    /// Specify the local IP of this machine, if multiple interfaces are available.
    #[clap(long)]
    local_ip: Option<Ipv4Addr>,

    /// The MQTT username, if needed
    #[clap(short, long)]
    username: Option<String>,

    /// The MQTT password, if needed
    #[clap(short, long)]
    password: Option<String>,

    /// The MQTT ID to use for this client, if needed
    #[clap(long, default_value = "mqtt-broadlink")]
    mqtt_id: String,

    /// The keepalive interval, in seconds.
    #[clap(long, default_value = "30")]
    keep_alive: u16,

    /// The operation timeout, in seconds.
    #[clap(long, default_value = "20")]
    operation_timeout: u16,

    /// Automatically connect to the broker
    #[clap(long)]
    auto_connect: bool,

    /// The MQTT broker used for publishing / subscribing to topics.
    mqtt_broker: String,
}

type DeviceMap = HashMap<Ipv4Addr, Device>;

#[tokio::main]
async fn main() {
    // Initialize needed vars
    let args = Args::parse();
    env_logger::init();

    info!("Starting broadlink-mqtt v{}...", env!("CARGO_PKG_VERSION"));

    // Get the devices
    let devices = get_devices(&args).expect("Could not find devices!");

    // Start an update thread on all the devices
    let mut threads: Vec<tokio::task::JoinHandle<_>> = vec![];
    for (_, device) in devices {
        let args_copy = args.clone();

        threads.push(tokio::spawn(async move {
            handle_device(&device, args_copy)
                .await
                .expect("Could not handle device!");
        }));
    }

    // Await on all of the threads
    for thread in threads {
        thread.await.expect("Could not run thread!");
    }
}

async fn handle_device(device: &Device, args: Args) -> Result<(), String> {
    let info = device.get_info();
    let sanitized_name = info
        .friendly_model
        .to_lowercase()
        .replace(" ", "-")
        .replace("/", ">");
    let mqtt_id = format!("{}-{}", args.mqtt_id.clone(), sanitized_name);

    // Construct the mqtt client
    let mut builder = Client::builder();
    builder
        .set_url_string(&args.mqtt_broker)
        .expect("Could not set MQTT broker URL!")
        .set_username(args.username.clone())
        .set_password(args.password.clone().map(|s| s.as_bytes().to_vec()))
        .set_client_id(Some(mqtt_id))
        .set_connect_retry_delay(Duration::from_secs(1))
        .set_keep_alive(KeepAlive::from_secs(args.keep_alive))
        .set_operation_timeout(Duration::from_secs(args.operation_timeout as u64))
        .set_automatic_connect(args.auto_connect);

    let mut mqtt_client = builder
        .build()
        .expect("Could not construct the MQTT client!");

    // Connect to the broker
    info!("Connecting to the MQTT broker at {}", &args.mqtt_broker);
    mqtt_client
        .connect()
        .await
        .expect("Could not connect to MQTT broker!");

    // Publish the device information
    let mut msg = Publish::new(
        get_path(&sanitized_name, &["info"]),
        info.friendly_type.into(),
    );
    msg.set_qos(QoS::AtLeastOnce);
    msg.set_retain(false);

    mqtt_client
        .publish(&msg)
        .await
        .expect("Could not publish to the broker!");

    // Set up a listener for the commands
    let command_subscription = Subscribe::new(vec![SubscribeTopic {
        qos: QoS::AtLeastOnce,
        topic_path: get_path(&sanitized_name, &["cmd"]),
    }]);
    mqtt_client
        .subscribe(command_subscription)
        .await
        .expect("Could not subscribe to command topic!")
        .any_failures()
        .expect("Failures encountered when subscribing to command topic!");

    // Handle events indefinitely
    loop {
        let response = mqtt_client.read_subscriptions().await;
        match response {
            Err(Error::Disconnected) => {
                return Err("Device was disconnected from the broker!".into())
            }
            Ok(r) => {
                // Consume the command
                let data = String::from_utf8(r.payload().to_vec())
                    .expect("Couldn't read subscription message!");

                info!("Parsing message => {:?}", data);
                let parts: Vec<&str> = data.split(" ").collect();
                let cmd = parts.get(0);
                let payload = parts.get(1);

                // Make sure that we skip non-cmds.
                if cmd == None || payload == None {
                    warn!(
                        "Skipping incomplete command for {}: {}",
                        &sanitized_name, &data
                    );
                    continue;
                }

                let unwrapped_cmd = *cmd.unwrap();
                let unwrapped_payload = *payload.unwrap();

                info!(
                    "Got command '{}' with payload => {}",
                    unwrapped_cmd, unwrapped_payload
                );
                match unwrapped_cmd {
                    "blast" => {
                        handle_blast(&mqtt_client, &device, &sanitized_name, &unwrapped_payload)
                            .await
                            .expect("Could not handle blast!")
                    }
                    "learn" => {
                        handle_learn(&mqtt_client, &device, &sanitized_name, &unwrapped_payload)
                            .await
                            .expect("Could not handle learn!")
                    }
                    _ => warn!(
                        "Skipping unknown command for {}: {}",
                        &sanitized_name, &unwrapped_cmd
                    ),
                }
            }
            Err(e) => warn!("Got unhandled error: {:?}", e),
        }
    }
}

/// Handles a blast command
async fn handle_blast(
    client: &Client,
    device: &Device,
    sanitized_name: &str,
    payload: &str,
) -> Result<(), String> {
    // Decode the payload into a hex array.
    let hex = hex::decode(payload);
    if let Err(e) = hex {
        warn!("Skipping invalid hex data: {:?}", e);
        return Ok(());
    }

    let hex = hex.unwrap();
    info!("Blasting payload {:?}", &hex);

    match device {
        Device::Remote { remote } => match remote.send_code(&hex) {
            Err(e) => {
                let err_msg = Publish::new(
                    get_path(&sanitized_name, &["blast_error"]),
                    e.to_string().into(),
                );
                client
                    .publish(&err_msg)
                    .await
                    .expect("Could not publish blast error!");

                return Ok(());
            }
            _ => info!("Blasted code successfully: {:?}", hex),
        },
        _ => {
            warn!("Device sent blast command, but is not a remote: {}", device);
            return Ok(());
        }
    };

    // Tell the MQTT broker that we successfully blasted
    let ok_msg = Publish::new(get_path(&sanitized_name, &["blast_status"]), "ok".into());
    client
        .publish(&ok_msg)
        .await
        .expect("Could not publish blast status!");

    return Ok(());
}

/// Handles a learn command
async fn handle_learn(
    client: &Client,
    device: &Device,
    sanitized_name: &str,
    payload: &str,
) -> Result<(), String> {
    // Only remotes can learn, so extract it here.
    let remote = match device {
        Device::Remote { remote } => remote,
        _ => {
            warn!("Device sent learn command, but is not a remote: {}", device);
            return Ok(());
        }
    };

    // Try to learn the code
    let code = match payload {
        "ir" => remote.learn_ir(),
        "rf" => remote.learn_rf(),
        _ => {
            warn!("Skipping invalid learn mode {}", payload);
            return Ok(());
        }
    };

    // Short out if no code was learned.
    if let Err(e) = code {
        warn!("Device did not find any code! {:?}", e);
        let err_msg = Publish::new(
            get_path(sanitized_name, &["code_error"]),
            e.to_string().into(),
        );
        client
            .publish(&err_msg)
            .await
            .expect("Could not publish code error message!");

        return Ok(());
    }

    // Convert the code into a hex string
    let hex_code = hex::encode(code.unwrap());

    // Publish the learned code
    let code_msg = Publish::new(get_path(sanitized_name, &["code"]), hex_code.into());
    client
        .publish(&code_msg)
        .await
        .expect("Could not send learned code!");

    return Ok(());
}

/// Returns a topic path for a given device and path structure.
fn get_path(sanitized_name: &str, paths: &[&str]) -> String {
    return format!("broadlink/{}/{}", sanitized_name, paths.join("/"));
}

/// Get devices through autodiscovery or manual clients
fn get_devices(args: &Args) -> Result<DeviceMap, String> {
    let mut result: DeviceMap = DeviceMap::new();

    // Auto discover devices, if enabled
    if args.auto_discover {
        info!("Autodiscovering devices...");
        let discovered = Device::list(args.local_ip).expect("Could not enumerate devices!");
        for device in discovered {
            let addr = device.get_info().address;

            info!("Discovered device at {} => {}", addr, device);
            result.insert(addr, device);
        }
    }

    // Add all of the static clients, if present
    for addr in &args.client {
        info!("Adding client at {}", addr);

        // Skip clients that we know of already
        if result.contains_key(&addr) {
            warn!("Skipping duplicate client {}", addr);
            continue;
        }

        let client = Device::from_ip(*addr, args.local_ip)
            .expect(format!("Could not add client {}!", addr).as_str());

        info!("Client added => {}", client);
        result.insert(*addr, client);
    }

    return Ok(result);
}
