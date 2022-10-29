use std::{io::Write, rc::Rc};

use clap::Parser;
#[allow(unused_variables, unused_imports)]
use jaac::navigator;
use jaac::utils::*;
use jaac::{
    cli::{Cli, Commands},
    db::SessionInfo,
};
use rpassword::read_password;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //         .expect("An Error Occured when trying to initialize client");
    // // let conf = mynav.get_mac_config_page_data().await?;
    // // let data = mynav.post_mac_config_page_data(conf).await?;
    // // print!("{:?}", data);
    // mynav.get_connected_devices().await?;
    let cli = Cli::parse();
    let (_, conn) = SessionInfo::new("".to_string(), "".to_string()).unwrap();
    let connection = Rc::new(conn);

    if let Some(args) = cli.args {
        use jaac::cli::PosArgs;
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

                let mut mynav =
                    navigator::JioPageNavigator::new(session.username.clone(), session.password.clone()).unwrap();
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
                let session = SessionInfo::retrieve(Some(connection));
                println!("{:?}", session);
                let user_ip_addr = get_local_ip_address();

                let mut nav =
                    navigator::JioPageNavigator::new(session.username, session.password).unwrap();

                let mac_conf = nav.get_mac_config_page_data().await?;
                let connected_devices = nav.get_connected_devices().await?;
                println!("mac config \n {:?}", mac_conf);
                println!("connected devices \n {:?}", connected_devices);
            }
            PosArgs::ResetPassword => (),
            PosArgs::Status => (),
        }
    }

    match &cli.commands {
        Some(command) => match command {
            Commands::User(user_args) => (),
            Commands::Ls(ls_args) => (),
            Commands::Thanos { snap } => (),
            Commands::Group(group_args) => (),
        },
        None => (),
    }

    Ok(())
}
