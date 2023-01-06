use std::collections::HashMap;
// use std::time::Duration;
use std::{io::Write, rc::Rc};
use clap::Parser;
use jaac::cli::Mode;
use jaac::db::{Group, User, UserIdentifier};
#[allow(unused_variables, unused_imports)]
use jaac::navigator;
use jaac::navigator::ConnectedDevice;
use jaac::types::Error;
use jaac::utils::*;
use jaac::{
    cli::{Cli, Commands, GroupSubCommand},
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
                Commands::User(user_args) => {
                    let mut nav = get_jiofi_navigator(Rc::clone(&connection))?;
                    let mut identifier: Option<UserIdentifier> = None;
                    if let Some(host_name) = &user_args.devicename {
                        identifier = Some(UserIdentifier::HostName(host_name.to_owned()));
                    } else if let Some(ip_address) = &user_args.ip {
                        identifier = Some(UserIdentifier::IpAddress(ip_address.to_owned()));
                    } else if let Some(mac_address) = &user_args.mac {
                        identifier = Some(UserIdentifier::MacAddress(mac_address.to_owned()));
                    }
                    let user_identifier = identifier.expect("Please input a valid identfier, either the ip, mac, device name of the user.");

                    let user =
                        User::retrieve(user_identifier.clone(), Some(Rc::clone(&connection)))
                            .unwrap_or_else(|| {
                                use tokio::runtime::Runtime;
                                let connected_devices = Runtime::new()
                                    .unwrap()
                                    .block_on(nav.get_connected_devices())
                                    .expect("An error occured while retrieving connected devices");
                                let mut filted_devices: Vec<ConnectedDevice> = connected_devices
                                    .into_iter()
                                    .filter(|device| {
                                        let res = match &user_identifier {
                                            UserIdentifier::HostName(host_name) => {
                                                *host_name == *device.Host_name
                                            }
                                            UserIdentifier::IpAddress(ip_address) => {
                                                *ip_address == *device.IP_address
                                            }
                                            UserIdentifier::MacAddress(mac_address) => {
                                                *mac_address == *device.MAC
                                            }
                                            _ => panic!("Huh?"),
                                        };
                                        res
                                    })
                                    .collect();

                                if filted_devices.len() == 0 {
                                    panic!("Sorry device was not found");
                                }

                                let device = filted_devices.pop().unwrap();

                                let user = User::from(device);
                                user
                            });

                    if user_args.block {
                        let block_group =
                            Group::retrieve("default_block_list", Some(Rc::clone(&connection)))
                                .unwrap_or_else(|| {
                                    let new_block_list =
                                        Group::new("default_block_list", Mode::Block)
                                            .sync(Rc::clone(&connection))
                                            .expect("An unexpected error occured");
                                    new_block_list
                                });
                        block_group.add_user(user, Rc::clone(&connection))?;
                    }
                }
                Commands::Ls(ls_args) => {
                    if ls_args.connected {
                        let mut nav = get_jiofi_navigator(Rc::clone(&connection))?;
                        let connected_devices = nav.get_connected_devices().await?;
                        let mut user_list: Vec<User> = vec![];

                        for device in connected_devices {
                            user_list.push(User::from(device))
                        }

                        User::print_user_list(user_list)?;
                    } else if ls_args.all {
                        let user_record = User::get_user_data(connection)?;
                        let users = User::get_all_users(user_record);
                        User::print_user_list(users)?;
                    } else if ls_args.block_list {
                    } else {
                    }
                }
                Commands::Group(group_args) => {
                    if let Some(sub_command) = &group_args.subcmd {
                        match sub_command {
                            GroupSubCommand::Create(group) => {
                                let group = Group::new(&group.name, group.mode);
                                group.sync(connection)?;
                            }
                            GroupSubCommand::Modify(group) => {}

                            GroupSubCommand::Remove(group) => {}
                        }
                    }
                }
                Commands::Thanos { snap } => (),
            }
        }
        None => (),
    }

    Ok(())
}
