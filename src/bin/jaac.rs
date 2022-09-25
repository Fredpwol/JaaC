use clap::Parser;
#[allow(unused_variables, unused_imports)]

use jaac::navigator;
use jaac::cli::{Cli};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut mynav =
    //     navigator::JioPageNavigator::new("freddthink".to_string(), "Hellno2358".to_string())
    //         .expect("An Error Occured when trying to initialize client");
    // // let conf = mynav.get_mac_config_page_data().await?; 
    // // let data = mynav.post_mac_config_page_data(conf).await?;
    // // print!("{:?}", data);
    // mynav.get_connected_devices().await?;
    let cli = Cli::parse();
    Ok(())
}
