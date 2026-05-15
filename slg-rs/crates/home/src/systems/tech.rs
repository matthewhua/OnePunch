//! 科技系统（TechSystem）
//!
//! 对应 Java 版 TechnologyFunction，管理科技研究、加速。
//! 数据存储在 p_data.technology_func（protobuf TechnologyDataFunction）。

use std::sync::Arc;
use anyhow::Result;
use prost::Message;
use tracing::info;

use super::PlayerSystem;
use proto::slg::{
    TechnologyDataFunction, TechnologyNode, TechnologyResearchQueue,
    TechnologyResearchRq, TechnologyResearchRs,
    TechnologySpeedUpRq, TechnologySpeedUpRs,
    TechnologyCancelRq, TechnologyCancelRs,
};
use shared::persistence::col;
use shared::event::GameEvent;
use shared::static_config::StaticConfig;

/// 科技系统
pub struct TechSystem {
    dirty: bool,
    /// 已研究的科技节点
    pub nodes: Vec<TechnologyNode>,
    /// 研究队列
    pub queue: Vec<TechnologyResearchQueue>,
}

impl TechSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            nodes: Vec::new(),
            queue: Vec::new(),
        }
    }

    /// 获取科技等级
    pub fn get_tech_level(&self, tech_id: i32) -> i32 {
        self.nodes.iter()
            .find(|n| n.technology_id == Some(tech_id))
            .and_then(|n| n.level)
            .unwrap_or(0)
    }

    /// 是否正在研究
    pub fn is_researching(&self) -> bool {
        !self.queue.is_empty()
    }

    /// tick 检测研究完成，返回触发的游戏事件列表
    pub fn check_research_complete(&mut self, role_id: i64, now_secs: i64) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let mut completed: Vec<TechnologyResearchQueue> = Vec::new();

        self.queue.retain(|q| {
            let done = q.complete_time.map(|t| t <= now_secs).unwrap_or(false);
            if done {
                completed.push(q.clone());
            }
            !done
        });

        for q in completed {
            let tech_id = q.technology_id.unwrap_or(0);
            let research_level = q.research_level.unwrap_or(1);

            // 更新或插入科技节点
            if let Some(node) = self.nodes.iter_mut().find(|n| n.technology_id == Some(tech_id)) {
                node.level = Some(research_level);
            } else {
                self.nodes.push(TechnologyNode {
                    technology_id: Some(tech_id),
                    level: Some(research_level),
                    stage: q.research_stage,
                });
            }
            self.dirty = true;

            events.push(GameEvent::TechResearchComplete {
                role_id,
                tech_id,
                new_level: research_level,
            });
        }

        events
    }

    /// 命令分发入口
    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        match cmd {
            4201 => self.cmd_research(payload, config),
            4203 => self.cmd_speed_up(payload, config),
            4205 => self.cmd_cancel(payload, config),
            _ => Err(anyhow::anyhow!("Unknown tech cmd: {}", cmd)),
        }
    }

    // ── 科技研究（cmd=4201）────────────────────────────────────────────────────

    fn cmd_research(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = TechnologyResearchRq::decode(payload)
            .map_err(|e| anyhow::anyhow!("Decode TechnologyResearchRq: {}", e))?;
        let tech_id = rq.technology_id;
        let stage = rq.stage;
        let research_type = rq.r#type; // 1=普通, 2=立即完成

        // 检查是否已在研究
        if self.queue.iter().any(|q| q.technology_id == Some(tech_id)) {
            return Err(anyhow::anyhow!("Tech {} already in queue", tech_id));
        }

        // 获取当前等级，计算下一级
        let cur_level = self.get_tech_level(tech_id);
        let next_level = cur_level + 1;

        // TODO: 检查资源消耗（s_tech_lv.upNeedResource）
        let now = chrono::Utc::now().timestamp();

        if research_type == 2 {
            // 立即完成：直接更新节点
            if let Some(node) = self.nodes.iter_mut().find(|n| n.technology_id == Some(tech_id)) {
                node.level = Some(next_level);
            } else {
                self.nodes.push(TechnologyNode {
                    technology_id: Some(tech_id),
                    level: Some(next_level),
                    stage: Some(stage),
                });
            }
            self.dirty = true;
            let events = vec![GameEvent::TechResearchComplete {
                role_id: 0, tech_id, new_level: next_level,
            }];
            return Ok((TechnologyResearchRs::default().encode_to_vec(), events));
        }

        // 普通研究：加入队列
        let research_time = config.tech.tech_levels.values()
            .find(|t| t.tech_id == tech_id && t.level == next_level)
            .map(|t| t.up_time as i64)
            .unwrap_or(60);

        self.queue.push(TechnologyResearchQueue {
            technology_id: Some(tech_id),
            research_level: Some(next_level),
            complete_time: Some(now + research_time),
            research_stage: Some(stage),
            ..Default::default()
        });
        self.dirty = true;

        Ok((TechnologyResearchRs::default().encode_to_vec(), vec![]))
    }

    // ── 研究加速（cmd=4203）────────────────────────────────────────────────────

    fn cmd_speed_up(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = TechnologySpeedUpRq::decode(payload)
            .map_err(|e| anyhow::anyhow!("Decode TechnologySpeedUpRq: {}", e))?;
        let tech_id = rq.technology_id;
        let speed_type = rq.r#type; // 1=道具加速, 2=立即完成, 3=道具一键加速

        if speed_type == 2 {
            // 立即完成：从队列移除并直接完成
            if let Some(pos) = self.queue.iter().position(|q| q.technology_id == Some(tech_id)) {
                let q = self.queue.remove(pos);
                let new_level = q.research_level.unwrap_or(1);
                if let Some(node) = self.nodes.iter_mut().find(|n| n.technology_id == Some(tech_id)) {
                    node.level = Some(new_level);
                } else {
                    self.nodes.push(TechnologyNode {
                        technology_id: Some(tech_id),
                        level: Some(new_level),
                        stage: q.research_stage,
                    });
                }
                self.dirty = true;
                let events = vec![GameEvent::TechResearchComplete {
                    role_id: 0, tech_id, new_level,
                }];
                return Ok((TechnologySpeedUpRs::default().encode_to_vec(), events));
            }
        } else {
            // TODO: 道具加速（减少 complete_time）
            if let Some(q) = self.queue.iter_mut().find(|q| q.technology_id == Some(tech_id)) {
                let reduce_secs = 300i64; // 简化：每次减少5分钟
                q.complete_time = q.complete_time.map(|t| (t - reduce_secs).max(0));
                self.dirty = true;
            }
        }

        Ok((TechnologySpeedUpRs::default().encode_to_vec(), vec![]))
    }

    // ── 取消研究（cmd=4205）────────────────────────────────────────────────────

    fn cmd_cancel(
        &mut self,
        payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        let rq = TechnologyCancelRq::decode(payload)
            .map_err(|e| anyhow::anyhow!("Decode TechnologyCancelRq: {}", e))?;
        let tech_id = rq.technology_id;

        self.queue.retain(|q| q.technology_id != Some(tech_id));
        self.dirty = true;

        // TODO: 返还部分资源
        Ok((TechnologyCancelRs::default().encode_to_vec(), vec![]))
    }

    fn to_proto(&self) -> TechnologyDataFunction {
        TechnologyDataFunction {
            node: self.nodes.clone(),
            queue: self.queue.clone(),
        }
    }
}

impl PlayerSystem for TechSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        let func = TechnologyDataFunction::decode(data)?;
        self.nodes = func.node;
        self.queue = func.queue;
        info!(techs = self.nodes.len(), queue = self.queue.len(), "TechSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.to_proto().encode_to_vec())
    }

    fn is_dirty(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn column_name(&self) -> &'static str { col::TECHNOLOGY }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        let (resp, _) = self.handle_command_with_events(cmd, payload, config)?;
        Ok(resp)
    }

    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<GameEvent>)> {
        TechSystem::handle_command(self, cmd, payload, config)
    }
}

impl shared::msg::ToFunctionClientBaseBytes for TechSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_type, func_tag};
        shared::msg::build_function_base_bytes_pub(func_type::TECHNOLOGY, func_tag::TECHNOLOGY, &self.to_proto())
    }
}
