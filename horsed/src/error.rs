use base64::DecodeError;
use displaydoc::Display as DocDisplay;
use russh::Error as SshError;
use sea_orm::DbErr;
use ssh_key::Error as SshKeyError;
use std::process::ExitStatusError;
use thiserror::Error as ThisError;

#[derive(Debug, DocDisplay, ThisError)]
pub enum Error {
    /// IO 错误: {0}
    IO(#[from] std::io::Error),
    /// 命令执行错误: {0}
    CmdError(#[from] ExitStatusError),
    /// Base64 解码错误: {0}
    Base64DecodeError(#[from] DecodeError),
    /// Ssh 错误: {0}
    SshError(#[from] SshError),
    /// SshKey 错误: {0}
    SshKeyError(#[from] SshKeyError),
    /// DB Error: {0}
    DbError(#[from] DbErr),
    /// 其他错误: {0}
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
