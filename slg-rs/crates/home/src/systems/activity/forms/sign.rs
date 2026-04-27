use anyhow::Result;
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
        // TODO: 从 PB 反序列化
        let decoded: Self = serde_json::from_slice(data)?;
        *self = decoded;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        // TODO: 序列化为 PB
        Ok(serde_json::to_vec(self)?)
    }

    fn to_pb(&self, activity_id: i32, form_id: i32) -> Result<proto::slg::ActivityFormPb> {
        use proto::slg::{ActivityFormPb, ActivityFormSignPb};
        use shared::msg::GameMessage;
        use bytes::Buf;
        use prost::Message;

        let mut form_pb = ActivityFormPb::default();
        form_pb.activity_id = Some(activity_id);
        form_pb.form_id = Some(form_id);
        form_pb.form_type = Some(self.form_type() as i32);

        // 构建 Sign 具体的 Extension 数据
        let mut sign_ext = ActivityFormSignPb::default();
        // 这里需要将内部状态映射到 sign_ext.sign_info
        // 示例：目前只有 sign_days
        sign_ext.sign_info.push(proto::slg::IntLong {
            v1: self.sign_days,
            v2: if self.signed_today { 3 } else { 2 }
        });

        // 编码 Extension 并存入 unknown_fields
        let mut buf = bytes::BytesMut::new();
        GameMessage::encode_extension(11, &sign_ext, &mut buf);
        form_pb.unknown_fields.extend_from_slice(&buf);

        Ok(form_pb)
    }


    fn on_daily_tick(&mut self, _activity: &ActivityData, _day_num: i32) {
        // 跨天重置今日签到状态
        self.signed_today = false;
    }
}
