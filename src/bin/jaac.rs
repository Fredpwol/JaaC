use std::collections::HashMap;
// use std::time::Duration;
use std::{io::Write, rc::Rc};

use clap::Parser;
use jaac::db::User;
#[allow(unused_variables, unused_imports)]
use jaac::navigator;
use jaac::navigator::ConnectedDevice;
use jaac::types::Error;
use jaac::utils::*;
use jaac::{
    cli::{Cli, Commands},
    db::SessionInfo,
};
use rpassword::read_password;
use rusqlite::Connection;
use tokio;

fn get_jiofi_navigator(connection: Rc<Connection>) -> Result<navigator::JioPageNavigator, Error> {
    let session = SessionInfo::retrieve(Some(connection));

    let nav = navigator::JioPageNavigator::new(session.username, session.password)?;
    Ok(nav)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    //         .expect("An Error Occured when trying to initialize client");
    // // let conf = mynav.get_mac_config_page_data().await?;
    // // let data = mynav.post_mac_config_page_data(conf).await?;
    // // print!("{:?}", data);
    // mynav.get_connected_devices().await?;
    let cli = Cli::parse();

    if let Some(args) = cli.args {
        use jaac::cli::PosArgs;
        let (_, conn) = SessionInfo::new("".to_string(), "".to_string()).unwrap();
        let connection = Rc::new(conn);
        match args {
            PosArgs::Browser => (),
            PosArgs::Login => {
                let mut username = String::new();

                print!("Please input your username > ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut username).unwrap();
                print!("Please input your password > ");
                std::io::stdout().flush().unwrap();
                let password = read_password().unwrap();

                let session = match SessionInfo::new(
                    username.strip_suffix("\n").unwrap().to_string(),
                    password,
                ) {
                    Ok((sess, _)) => sess,
                    Err(error) => {
                        panic!(
                            "Sorry an error occured when creating a session {}",
                            error.to_string()
                        );
                    }
                };

                let mut mynav = navigator::JioPageNavigator::new(
                    session.username.clone(),
                    session.password.clone(),
                )
                .unwrap();
                match mynav.login().await {
                    Ok(()) => println!("Login successfully"),
                    Err(error) => {
                        panic!(
                            "Invalid username or password, please try again!, {}",
                            error.to_string()
                        );
                    }
                }

                session.create(Rc::clone(&connection)).unwrap();
            }
            PosArgs::Purge => {
                let mut nav = get_jiofi_navigator(Rc::clone(&connection))?;

                let connected_devices = nav.get_connected_devices().await?;

                let mut host_device: Option<ConnectedDevice> = None;

                let user_ip_addr = get_local_ip_address();

                for device in connected_devices {
                    if device.IP_address == user_ip_addr {
                        host_device = Some(device);
                    }
                }

                let mac_conf = nav.get_mac_config_page_data().await?;

                match host_device {
                    Some(device) => {
                        let mut payload = vec![];

                        let mut mac_rule: HashMap<String, String> = HashMap::new();
                        mac_rule.insert("rule_address".to_string(), device.MAC);
                        mac_rule.insert("rule_name".to_string(), device.Host_name);
                        payload.push(MacFilterOptionType::MacRuleTable(vec![mac_rule]));

                        use jaac::navigator::MacFilterOptionType;
                        for mac_option in mac_conf {
                            match mac_option {
                                MacFilterOptionType::MacRules(mut opts) => {
                                    opts.insert("MACFILTER_MODE".to_string(), "1".to_string());
                                    opts.insert("MACFILTER_ENABLE".to_string(), "1".to_string());
                                    payload.push(MacFilterOptionType::MacRules(opts));
                                }
                                _ => (),
                            };
                        }
                        nav.post_mac_config_page_data(payload).await?;
                        println!("Done");
                    }
                    None => {
                        panic!();
                    }
                }
            }
            PosArgs::ResetPassword => (),
            PosArgs::Status => (),
            PosArgs::Clean => {
                use jaac::navigator::MacFilterOptionType;

                let mut nav = get_jiofi_navigator(connection)?;
                let mut payload = HashMap::new();

                let mac_rule = HashMap::from([
                    ("MACFILTER_MODE".to_string(), "0".to_string()),
                    ("MACFILTER_ENABLE".to_string(), "0".to_string()),
                ]);

                payload.insert("rule_table", MacFilterOptionType::MacRuleTable(vec![]));
                payload.insert("mac_rules", MacFilterOptionType::MacRules(mac_rule));

                nav.update_setting(payload, true).await?;
                println!("Cleaned!!");
            }
        }
    }

    match &cli.commands {
        Some(command) => {
            let (_, conn) = SessionInfo::new("".to_string(), "".to_string()).unwrap();
            let connection = Rc::new(conn);
            match command {
                Commands::User(user_args) => (),
                Commands::Ls(ls_args) => {
                    if ls_args.connected {
                        let mut nav = get_jiofi_navigator(Rc::clone(&connection))?;
                        let connected_devices = nav.get_connected_devices().await?;
                        let mut user_list: Vec<User> = vec![];

                        for device in &connected_devices {
                            user_list.push(User::from(device))
                        }

                        User::print_user_list(user_list)?;
                    }
                }
                Commands::Thanos { snap } => (),
                Commands::Group(group_args) => (),
            }
        }
        None => (),
    }

    Ok(())
}
