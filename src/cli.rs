use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

///Advanced access configuration for the jiofi internet router
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(ArgGroup::new("cmd").args(&["purge", "reset-password", "login", "browser", "status"])))]
pub struct Cli {

    #[clap(subcommand)]
    pub commands: Option<Commands>,

    #[clap(value_parser, action)]
    pub purge: bool,

    #[clap(value_parser, action)]
    pub reset_password: bool,

    #[clap(value_parser, action)]
    pub login: bool,

    #[clap(value_parser, action)]
    pub browser: bool,

    #[clap(value_parser, action)]
    pub status: bool,

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
    pub all: Option<String>,

    #[clap(long, short)]
    pub connected: Option<String>,

    #[clap(long, short)]
    pub block_list: Option<String>,
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    Allow,
    Block,
}

#[derive(Args)]
pub struct GroupCreateCommand {
    #[clap(value_parser)]
    pub name: String,

    #[clap(long, short, arg_enum)]
    pub mode: Mode,
}

#[derive(Args)]
pub struct GroupModifyCommand {
    #[clap(value_parser)]
    pub group_name: String,

    #[clap(long, short, requires = "group-name")]
    pub name: Option<String>,

    #[clap(long, short, arg_enum, requires = "group-name")]
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
