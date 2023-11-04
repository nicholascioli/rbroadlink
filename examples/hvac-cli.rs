use rbroadlink::{traits::DeviceTrait, Device};
use std::env;

#[derive(PartialEq)]
enum RunMode {
    Help,
    Info,
    Toggle,
    TurnOn,
    TurnOff,
}

fn main() {
    let argument = env::args().nth(1);
    let run_mode = if let Some(arg) = argument {
        match &arg[..] {
            "info" => RunMode::Info,
            "toggle" => RunMode::Toggle,
            "on" => RunMode::TurnOn,
            "off" => RunMode::TurnOff,
            _ => RunMode::Help,
        }
    } else {
        RunMode::Help
    };

    if run_mode == RunMode::Help {
        println! {"No arguments given, possible choices:\n"};
        println! {"info      show air conditioner state"};
        println! {"on        power ON air conditioner"};
        println! {"off       power OFF air conditioner"};
        println! {"toggle    toggle power state"};
        return;
    };

    println!(">>> autodiscovering broadlink devices...");
    let discovered = Device::list(None).expect("Could not enumerate devices!");
    for device in discovered {
        println!(">>> device authentication ...");
        let addr = device.get_info().address;
        println!(">>> device at {} => {}", addr, device);

        let hvac = match device {
            Device::Hvac { hvac } => hvac,
            _ => {
                return;
            }
        };
        if run_mode == RunMode::Info {
            println!(">>> get_info");
            let ac_info = hvac.get_info().unwrap();
            println!("Current power state: {}", ac_info.power);
            println!("Ambient temperature: {:.1}", ac_info.get_ambient_temp());
        } else {
            println!(">>> get_state");
            let mut state = hvac.get_state().unwrap();
            println!("Current state: {:?}", state);

            // Setting desired mode according to command line argument
            if run_mode == RunMode::Toggle {
                state.power = !state.power;
            } else if run_mode == RunMode::TurnOn {
                state.power = true;
            } else if run_mode == RunMode::TurnOff {
                state.power = false;
            }

            println!(">>> set_state");
            let response = hvac.set_state(&mut state).unwrap();
            println!(">>> device response {:02x?}", response);
        }
    }
}
