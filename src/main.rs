use clap::{Arg, Command};
use inquire::Select;
use regex::Regex;
use std::process::{exit, Command as ProcessCommand};

fn cli() -> Command {
    Command::new("sniff")
        .about("CLI for listing services listening on a port and killing it")
        .arg(
            Arg::new("port")
                .help("The port to sniff")
                .required(true)
                .index(1),
        )
}

fn main() {
    let matches = cli().get_matches();
    let port_str = matches
        .get_one::<String>("port")
        .expect("Port argument missing");

    match port_str.parse::<u16>() {
        Ok(port) => {
            let output = ProcessCommand::new("lsof")
                .arg("-i")
                .arg(format!("TCP:{}", port))
                .output()
                .unwrap_or_else(|_| panic!("Failed to execute lsof command"));

            let output_str = String::from_utf8_lossy(&output.stdout);
            let re =
                Regex::new(r"^(?P<command>\S+)\s+(?P<pid>\d+)").expect("Failed to compile regex");

            let mut choices: Vec<(String, String)> = Vec::new();

            for line in output_str.lines() {
                if let Some(caps) = re.captures(line) {
                    let process_info =
                        format!("Process: {:<15} PID: {}", &caps["command"], &caps["pid"]);

                    choices.push((process_info, caps["pid"].to_string()));
                }
            }

            if choices.is_empty() {
                println!("No processes found on port {}", port);
                return;
            }

            let process_choices: Vec<_> = choices.iter().map(|(info, _)| info.as_str()).collect();
            let ans = Select::new("Choose a process to kill", process_choices).prompt();

            match ans {
                Ok(choice) => {
                    let &(_, ref pid) = choices
                        .iter()
                        .find(|(info, _)| info == choice)
                        .expect("Process not found");

                    println!("Sending SIGKILL to: {}", choice);
                    ProcessCommand::new("kill")
                        .arg(pid)
                        .output()
                        .unwrap_or_else(|_| panic!("Failed to execute kill command"));
                }
                Err(_) => println!("There was an error, please try again"),
            }
        }
        Err(_) => {
            eprintln!("Error: Port must be a valid number");
            exit(1);
        }
    }
}
