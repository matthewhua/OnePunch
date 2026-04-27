use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WalEntry {
    /// 部队开始行军
    MarchStart {
        key: i32,
        origin: i32,
        goal: i32,
        start_time: i64,
        end_time: i64,
    },
    /// 部队转移到另一个 Sector
    TroopTransfer {
        key: i32,
        target_sector: i32,
    },
    /// 资源变动
    ResourceUpdate {
        role_id: i64,
        pos: i32,
        res_type: i32,
        amount: i64,
    },
    /// 检查点（表示之前的日志已经安全存盘到数据库，可以截断）
    Checkpoint {
        sequence: u64,
    },
}

pub struct WriteAheadLog {
    file: File,
    path: String,
    sequence: u64,
}

impl WriteAheadLog {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)
            .await?;
            
        Ok(Self {
            file,
            path: path_str,
            sequence: 0,
        })
    }

    /// 追加一条日志
    pub async fn append(&mut self, entry: &WalEntry) -> Result<u64> {
        self.sequence += 1;
        let data = bincode::serialize(entry)?;
        let len = data.len() as u32;
        
        // 写入长度 + 数据
        self.file.write_all(&len.to_le_bytes()).await?;
        self.file.write_all(&data).await?;
        
        // 强制刷盘 (根据性能要求可以调整为定期刷盘)
        self.file.flush().await?;
        
        Ok(self.sequence)
    }

    /// 从日志中恢复数据
    pub async fn recover(&mut self) -> Result<Vec<WalEntry>> {
        let mut entries = Vec::new();
        let mut file = File::open(&self.path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        
        let mut cursor = 0;
        while cursor + 4 <= buffer.len() {
            let len = u32::from_le_bytes(buffer[cursor..cursor+4].try_into()?) as usize;
            cursor += 4;
            
            if cursor + len > buffer.len() {
                error!("WAL corrupted: unexpected end of file");
                break;
            }
            
            let entry: WalEntry = bincode::deserialize(&buffer[cursor..cursor+len])?;
            entries.push(entry);
            cursor += len;
        }
        
        info!("WAL recovered {} entries from {}", entries.len(), self.path);
        Ok(entries)
    }

    /// 截断日志（存盘后执行）
    pub async fn truncate(&mut self) -> Result<()> {
        // 简单处理：重新打开文件并清空
        self.file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await?;
        self.sequence = 0;
        Ok(())
    }
}
