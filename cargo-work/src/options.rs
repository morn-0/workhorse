pub use crate::mac::build::Build;
pub use crate::mac::check::Check;
pub use crate::mac::install::Install;
pub use crate::mac::run::Run;
pub use crate::mac::test::Test;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// 命令行参数
#[derive(Clone, Debug, Parser)]
#[clap(
    version,
    name = "cargo-work",
    styles = cargo_options::styles(),
    disable_help_subcommand = true,
)]
pub struct Cli {
    #[clap(short, long, help = "显示详细信息")]
    pub verbose: bool,

    #[clap(flatten)]
    pub horse: HorseOptions,

    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Default, Clone, Debug, Args)]
pub struct HorseOptions {
    #[clap(short, long = "ssh-key", help = "指定私钥文件路径")]
    pub key: Option<PathBuf>,
    #[clap(
        long = "repo",
        help = "指定仓库地址, 例如: ssh://127.0.0.1:2222/uuhan/workhorse"
    )]
    pub repo: Option<String>,
    #[clap(long = "repo-name", help = "指定仓库名称, 例如: [/]uuhan/workhorse")]
    pub repo_name: Option<String>,
}

#[derive(Clone, Debug, Subcommand)]
#[command(version, display_order = 1)]
pub enum Commands {
    #[command(name = "work", about = "cargo work")]
    Work(WorkOptions),
    #[command(flatten)]
    Cargo(Options),
}

#[derive(Clone, Debug, Parser)]
pub struct WorkOptions {
    #[clap(flatten)]
    pub horse: HorseOptions,

    #[clap(subcommand)]
    pub commands: Options,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
#[command(version, display_order = 1)]
pub enum Options {
    #[command(name = "build", alias = "b", about = "构建项目")]
    Build(Build),
    #[command(name = "check", alias = "c", about = "检查项目")]
    Check(Check),
    #[command(name = "install", alias = "i", about = "安装程序")]
    Install(Install),
    #[command(name = "test", alias = "t", about = "测试项目")]
    Test(Test),
    #[command(name = "run", alias = "r", about = "运行程序")]
    Run(Run),
    #[command(name = "just", alias = "j", about = "运行 just 任务")]
    Just(JustOptions),
    #[command(name = "push", alias = "p", about = "推送代码到远程仓库")]
    Push,
    #[command(name = "pull", alias = "l", about = "拉取编译资产")]
    Pull,
}

#[derive(Clone, Debug, Args)]
pub struct JustOptions {
    #[clap(short, long, help = "指定配置文件")]
    pub file: Option<String>,
    pub command: Option<String>,
    #[clap(flatten)]
    pub horse: HorseOptions,
}
