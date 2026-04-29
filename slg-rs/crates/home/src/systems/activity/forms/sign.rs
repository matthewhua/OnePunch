use anyhow::Result;
use prost::Message;
use bytes::BytesMut;
use crate::systems::activity::types::ActivityFormType;
use crate::systems::activity::model::{PersonalForm, ActivityData};

/// 签到玩法表单
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SignForm {
    /// 累计签到天数
    pub sign_days: i32,
    /// 今日是否已签到
    pub signed_today: bool,
    /// 上次签到时间戳
    pub last_sign_time: i64,
}

impl PersonalForm for SignForm {
    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::Sign
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        // TODO: 从 PB 反序列化（目前用 JSON 占位）
        let decoded: Self = serde_json::from_slice(data)?;
        *self = decoded;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        // TODO: 序列化为 PB（目前用 JSON 占位）
        Ok(serde_json::to_vec(self)?)
    }

    fn to_pb(&self, activity_id: i32, form_id: i32) -> Result<proto::slg::ActivityFormPb> {
        use proto::slg::{ActivityFormPb, ActivityFormSignPb, IntLong};
        use shared::msg::GameMessage;

        // 1. 构建 ActivityFormPb 基础字段
        let mut form_pb = ActivityFormPb::default();
        form_pb.activity_id = Some(activity_id);
        form_pb.form_id = Some(form_id);
        form_pb.form_type = Some(self.form_type() as i32);

        // 2. 构建 Sign Extension 数据
        let mut sign_ext = ActivityFormSignPb::default();
        // v1=第几天, v2=签到状态(0-不可签, 2-可领奖, 3-已领奖)
        for day in 1..=self.sign_days {
            sign_ext.sign_info.push(IntLong { v1: day, v2: 3 }); // 已签到
        }
        // 今日未签到则标记为可签
        if !self.signed_today {
            sign_ext.sign_info.push(IntLong {
                v1: self.sign_days + 1,
                v2: 2,
            });
        }

        // 3. 将 ActivityFormPb 基础字段 + Sign Extension 手动编码为字节流
        //    ActivityFormSignPb 的 extension tag = 11（proto 文件中 ext = 11）
        let mut buf = BytesMut::new();
        form_pb.encode(&mut buf)?;
        GameMessage::encode_extension(11, &sign_ext, &mut buf);

        // 4. 重新解码为 ActivityFormPb（extension 数据会作为未知字段保留，客户端可正确解析）
        let final_pb = ActivityFormPb::decode(buf.as_ref())?;
        Ok(final_pb)
    }

    fn on_daily_tick(&mut self, _activity: &ActivityData, _day_num: i32) {
        // 跨天重置今日签到状态
        self.signed_today = false;
    }
}
