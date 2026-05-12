use anyhow::Result;
use bytes::BytesMut;
use prost::Message;

use crate::systems::activity::model::{ActivityData, PersonalForm};
use crate::systems::activity::types::ActivityFormType;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SignForm {
    pub sign_days: i32,
    pub signed_today: bool,
    pub last_sign_time: i64,
}

impl SignForm {
    pub fn sign(&mut self, day_num: Option<i32>, now: i64) -> Result<()> {
        if self.signed_today {
            anyhow::bail!("activity sign already claimed today");
        }

        let expected = self.sign_days + 1;
        if let Some(day) = day_num {
            if day != expected {
                anyhow::bail!("invalid sign day {}, expected {}", day, expected);
            }
        }

        self.sign_days = expected;
        self.signed_today = true;
        self.last_sign_time = now;
        Ok(())
    }
}

impl PersonalForm for SignForm {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn form_type(&self) -> ActivityFormType {
        ActivityFormType::Sign
    }

    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        *self = serde_json::from_slice(data)?;
        Ok(())
    }

    fn serialize(&self, _save_db: bool) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn to_pb(&self, activity_id: i32, form_id: i32) -> Result<proto::slg::ActivityFormPb> {
        use proto::slg::{ActivityFormPb, ActivityFormSignPb, IntLong};
        use shared::msg::GameMessage;

        let mut form_pb = ActivityFormPb::default();
        form_pb.activity_id = Some(activity_id);
        form_pb.form_id = Some(form_id);
        form_pb.form_type = Some(self.form_type() as i32);

        let mut sign_ext = ActivityFormSignPb::default();
        for day in 1..=self.sign_days {
            sign_ext.sign_info.push(IntLong { v1: day, v2: 3 });
        }
        if !self.signed_today {
            sign_ext.sign_info.push(IntLong {
                v1: self.sign_days + 1,
                v2: 2,
            });
        }

        let mut buf = BytesMut::new();
        form_pb.encode(&mut buf)?;
        GameMessage::encode_extension(11, &sign_ext, &mut buf);
        Ok(ActivityFormPb::decode(buf.as_ref())?)
    }

    fn on_daily_tick(&mut self, _activity: &ActivityData, _day_num: i32) {
        self.signed_today = false;
    }
}
