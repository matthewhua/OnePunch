use std::collections::HashMap;
use anyhow::{anyhow, Result};
use super::PlayerSystem;

pub mod types;
pub mod model;
pub mod lifecycle;
pub mod forms;
pub mod settle;

use model::{PersonalActivity, ActivityPersistent, PersonalForm};
use forms::{sign::SignForm, task::TaskForm, score::ScoreForm, supreme_lord::SupremeLordForm};

/// 玩家活动系统（对应 Java ActivityFunction）
pub struct ActivitySystem {
    /// 玩家当前参与的所有活动数据：activityId -> PersonalActivity
    pub activities: HashMap<i32, PersonalActivity>,
    /// 跨赛季持久化数据
    pub persistent: ActivityPersistent,
    /// 脏数据标记
    pub dirty: bool,
}

impl ActivitySystem {
    pub fn new() -> Self {
        Self {
            activities: HashMap::new(),
            persistent: ActivityPersistent::default(),
            dirty: false,
        }
    }

    /// 活动协议命令分发入口
    pub fn handle_command(&mut self, cmd: u32, payload: &[u8]) -> Result<Vec<u8>> {
        match cmd {
            8001 => self.get_activity_func_data(),
            8007 => self.activity_sign(payload),
            // ... 后续根据玩法逐渐补充
            _ => Err(anyhow!("未知的活动命令号: {}", cmd)),
        }
    }

    fn get_activity_func_data(&mut self) -> Result<Vec<u8>> {
        // TODO: 返回全量活动数据 PB
        Ok(vec![])
    }

    fn activity_sign(&mut self, _payload: &[u8]) -> Result<Vec<u8>> {
        // TODO: 处理签到逻辑
        Ok(vec![])
    }
}

impl PlayerSystem for ActivitySystem {
    fn load_from_bin(&mut self, _data: &[u8]) -> Result<()> {
        // TODO: 使用 Prost 解析 ActivityFunction PB
        // 这里需要处理 proto2 的 extension 映射逻辑
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        // TODO: 序列化为二进制
        Ok(vec![])
    }
}

/// 辅助函数：由于 proto2 extension 不被 prost 直接支持，此函数用于手动分发 Extension 字段编解码
#[allow(dead_code)]
fn decode_form_extension(form_type: types::ActivityFormType, raw_bytes: &[u8]) -> Result<Box<dyn model::PersonalForm>> {
    let mut form: Box<dyn PersonalForm> = match form_type {
        types::ActivityFormType::Sign => Box::new(SignForm::default()),
        types::ActivityFormType::Task => Box::new(TaskForm::default()),
        types::ActivityFormType::ScoreAward => Box::new(ScoreForm::default()),
        types::ActivityFormType::SupremeLord => Box::new(SupremeLordForm::default()),
        // ... 继续补充
        _ => return Err(anyhow!("不支持的玩法类型: {:?}", form_type)),
    };
    
    form.deserialize(raw_bytes)?;
    Ok(form)
}
