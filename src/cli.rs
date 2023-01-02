use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum PosArgs {
    Purge,
    ResetPassword,
    Login,
    Browser,
    Status,
    Clean
}
///Advanced access configuration for the jiofi internet router
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {

    #[clap(value_enum)]
    pub args: Option<PosArgs>,

    #[clap(subcommand)]
    pub commands: Option<Commands>,

}

#[derive(Subcommand)]
pub enum Commands {
    User(UserCommand),
    Thanos {
        #[clap(value_parser, action)]
        snap: bool,
    },
    Ls(LsCommand),
    Group(GroupCommand),
}

#[derive(Args)]
#[clap(group(ArgGroup::new("id").required(true).args(&["username", "ip", "mac"])))]
pub struct UserCommand {
    #[clap(short, long)]
    pub username: Option<String>,

    #[clap(short, long)]
    pub ip: Option<String>,

    #[clap(short, long)]
    pub mac: Option<String>,

    #[clap(value_parser, group = "ops", requires = "id", action)]
    pub block: bool,

    #[clap(value_parser, group = "ops", requires = "id", action)]
    pub allow: bool,

    #[clap(value_parser, group = "ops", requires = "id")]
    pub deny: Option<String>,
}

#[derive(Args)]
#[clap(group(ArgGroup::new("options").required(true).args(&["all", "connected", "block-list"])))]
pub struct LsCommand {
    #[clap(long, short)]
    pub all: bool,

    #[clap(long, short, action)]
    pub connected: bool,

    #[clap(long, short, action)]
    pub block_list: bool,
}

#[derive(Args)]
#[clap(group(ArgGroup::new("id").args(&["username", "ip", "mac"])))]
pub struct GroupCommand {
    #[clap(value_parser)]
    pub name: Option<String>,

    #[clap(value_parser, action)]
    pub clean: bool,

    #[clap(value_parser, group = "ops", requires = "name")]
    pub deny: Option<String>,

    #[clap(value_parser, group = "ops", requires = "name", action)]
    pub delete: bool,

    #[clap(value_parser, group = "ops", requires = "name")]
    pub add: Option<String>,

    #[clap(subcommand)]
    pub subcmd: Option<GroupSubCommand>,
}

#[derive(Subcommand)]
pub enum GroupSubCommand {
    Create(GroupCreateCommand),
    Remove(GroupRemoveCommand),
    Modify(GroupModifyCommand),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    Allow = 1,
    Block = 0,
}

#[derive(Args)]
pub struct GroupCreateCommand {
    #[clap(value_parser)]
    pub name: String,

    #[clap(long, short, value_enum)]
    pub mode: Mode,
}

#[derive(Args)]
pub struct GroupModifyCommand {
    #[clap(value_parser)]
    pub group_name: String,

    #[clap(long, short, requires = "group-name")]
    pub name: Option<String>,

    #[clap(long, short, value_enum, requires = "group-name")]
    pub mode: Option<Mode>,
}

#[derive(Args)]
#[clap(group(ArgGroup::new("id").required(true).args(&["username", "ip", "mac"])))]
pub struct GroupRemoveCommand {
    #[clap(short, long)]
    pub username: Option<String>,

    #[clap(short, long)]
    pub ip: Option<String>,

    #[clap(short, long)]
    pub mac: Option<String>,
}
