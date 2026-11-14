// 功能：在打开数据库前独占当前数据目录，避免重复启动修改正在下载的任务和会话；时间：2026-11-14；作者：lq。
use std::{fs::{File, OpenOptions}, io, path::Path};

pub struct InstanceGuard { _file: File }

impl InstanceGuard {
    // 功能：锁由操作系统随进程退出释放，崩溃不留下需要手工删除的占用标记；时间：2026-11-14；作者：lq。
    pub fn acquire(directory: &Path) -> io::Result<Option<Self>> {
        let file = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(directory.join("sab.lock"))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(error)) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // 功能：独立文件句柄不能同时占用，释放后旧锁文件不影响重启；时间：2026-11-14；作者：lq。
    #[test]
    fn guard_prevents_duplicate_and_releases_on_drop() {
        let directory = tempfile::tempdir().unwrap();
        let guard = InstanceGuard::acquire(directory.path()).unwrap().unwrap();
        assert!(InstanceGuard::acquire(directory.path()).unwrap().is_none());
        drop(guard);
        assert!(InstanceGuard::acquire(directory.path()).unwrap().is_some());
    }
}
