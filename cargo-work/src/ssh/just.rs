use super::HorseClient;
use super::*;
use crate::options::JustOptions;
use anyhow::Context;
use anyhow::Result;
use git2::Repository;
use russh::ChannelMsg;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn run(sk: &Path, options: JustOptions) -> Result<()> {
    let repo = Repository::discover(".")?;
    let head = repo.head()?;

    let repo_name = if let Some(repo_name) = find_repo_name(&options.horse) {
        repo_name
    } else {
        // 无法从参数获取 repo_name, 尝试从 git remote 获取
        // 默认远程仓库为 horsed,
        // 格式: ssh://git@192.168.10.62:2222/<ns>/<repo_name>
        let Some(horsed) = find_remote(&repo) else {
            return Err(anyhow::anyhow!("找不到 horsed 远程仓库!"));
        };

        horsed
            .url()
            .and_then(extract_repo_name)
            .context("获取 horsed 远程仓库 URL 失败")?
    };

    let host = if let Ok(host) = std::env::var("HORSED") {
        host.parse()
            .context(format!("解析环境变量 HORSED 失败: {host}"))?
    } else if let Some(host) = find_host(&options.horse) {
        host
    } else {
        let Some(horsed) = find_remote(&repo) else {
            return Err(anyhow::anyhow!("找不到 horsed 远程仓库!"));
        };

        horsed
            .url()
            .and_then(extract_host)
            .context("获取 horsed 远程仓库 HOST 失败")?
    };

    let branch = head
        .shorthand()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "master".to_owned());
    let command = options.command.unwrap_or("default".to_string());

    #[cfg(feature = "use-system-ssh")]
    {
        let mut cmd = super::run_system_ssh(
            sk,
            format!("SetEnv REPO={} BRANCH={}", repo_name, branch),
            "just",
            host,
            command.clone(),
        );
        let mut ssh = cmd.spawn()?;
        let mut stdout = ssh.stdout.take().unwrap();
        let mut out = tokio::io::stdout();

        while let Ok(len) = tokio::io::copy(&mut stdout, &mut out).await {
            if len == 0 {
                break;
            }
        }
    }

    #[cfg(not(feature = "use-system-ssh"))]
    {
        let mut ssh = HorseClient::connect(sk, "just", host).await?;
        let mut channel = ssh.channel_open_session().await?;
        channel.set_env(true, "REPO", repo_name).await?;
        channel.set_env(true, "BRANCH", branch).await?;
        channel.exec(true, command.as_bytes()).await?;

        let mut code = None;
        let mut stdout = tokio::io::stdout();

        loop {
            // There's an event available on the session channel
            let Some(msg) = channel.wait().await else {
                break;
            };
            match msg {
                ChannelMsg::Success => {}

                // Write data to the terminal
                ChannelMsg::Data { ref data } => {
                    stdout.write_all(data).await?;
                    stdout.flush().await?;
                }
                // The command has returned an exit code
                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }

                ChannelMsg::ExtendedData { ref data, ext } => {
                    stdout.write_all(data).await?;
                    stdout.flush().await?;
                }

                ChannelMsg::Eof => {
                    break;
                }
                e => {}
            }
        }

        ssh.close().await?;
        code.context("program did not exit cleanly")?;
    }

    Ok(())
}
