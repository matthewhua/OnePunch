use anyhow::Result;
use prost::Message;
use proto::slg::{
    BaseMailPb, DelMailRq, DelMailRs, GetMailListRs, GetPersonalMailByIdRq, GetPersonalMailByIdRs,
    LockOrNotPersonalMailRq, LockOrNotPersonalMailRs, MailFunction, MailShowPb, ReadAllMailRq,
    ReadAllMailRs, TwoInt,
};
use shared::persistence::col;
use tracing::info;

use super::PlayerSystem;

const MAIL_TYPE_PERSONAL: i32 = 0;
const MAIL_STATUS_UNREAD: i32 = 1;
const MAIL_STATUS_READ: i32 = 2;
const MAIL_STATUS_UNREAD_WITH_AWARD: i32 = 3;
const MAIL_STATUS_READ_WITH_AWARD: i32 = 4;

pub struct MailSystem {
    dirty: bool,
    pub mails: Vec<BaseMailPb>,
}

impl MailSystem {
    pub fn new() -> Self {
        Self {
            dirty: false,
            mails: Vec::new(),
        }
    }

    pub fn add_personal_mail(&mut self, mut mail: BaseMailPb) -> i32 {
        mail.r#type = MAIL_TYPE_PERSONAL;
        if mail.key_id.is_none() || mail.key_id == Some(0) {
            mail.key_id = Some(self.next_key_id());
        }
        if mail.status.is_none() {
            mail.status = Some(initial_status(&mail));
        }
        self.mails.push(mail);
        self.dirty = true;
        self.mails
            .last()
            .and_then(|mail| mail.key_id)
            .unwrap_or_default()
    }

    fn next_key_id(&self) -> i32 {
        self.mails
            .iter()
            .filter_map(|mail| mail.key_id)
            .max()
            .unwrap_or_default()
            .saturating_add(1)
    }

    fn personal_mail_mut(&mut self, key_id: i32) -> Option<&mut BaseMailPb> {
        self.mails
            .iter_mut()
            .find(|mail| mail.key_id == Some(key_id) && mail.r#type == MAIL_TYPE_PERSONAL)
    }

    fn personal_mail(&self, key_id: i32) -> Option<&BaseMailPb> {
        self.mails
            .iter()
            .find(|mail| mail.key_id == Some(key_id) && mail.r#type == MAIL_TYPE_PERSONAL)
    }

    fn cmd_get_mail_list(&self) -> Result<Vec<u8>> {
        let mut mail_pb: Vec<MailShowPb> = self.mails.iter().map(mail_show_from_base).collect();
        mail_pb.sort_by(|a, b| b.time.unwrap_or_default().cmp(&a.time.unwrap_or_default()));
        Ok(GetMailListRs { mail_pb }.encode_to_vec())
    }

    fn cmd_get_personal_mail_by_id(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq = GetPersonalMailByIdRq::decode(payload)?;
        let mut changed = false;
        let mail_pb = self.personal_mail_mut(rq.key_id).map(|mail| {
            if mark_mail_read(mail) {
                changed = true;
            }
            mail.clone()
        });
        if changed {
            self.dirty = true;
        }
        Ok(GetPersonalMailByIdRs { mail_pb }.encode_to_vec())
    }

    fn cmd_read_all_mail(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq = ReadAllMailRq::decode(payload)?;
        let mut personal_mail = Vec::new();

        for key in rq.key_id {
            if key.v2 != MAIL_TYPE_PERSONAL {
                continue;
            }
            if let Some(mail) = self.personal_mail_mut(key.v1) {
                let changed = mark_mail_read(mail);
                let status = mail.status.unwrap_or_default();
                self.dirty |= changed;
                personal_mail.push(TwoInt {
                    v1: key.v1,
                    v2: status,
                });
            }
        }

        Ok(ReadAllMailRs {
            personal_mail,
            ..Default::default()
        }
        .encode_to_vec())
    }

    fn cmd_del_mail(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq = DelMailRq::decode(payload)?;
        let before = self.mails.len();
        self.mails.retain(|mail| {
            let Some(key_id) = mail.key_id else {
                return true;
            };
            !rq.key_id.iter().any(|key| {
                key.v1 == key_id
                    && key.v2 == mail.r#type
                    && mail.r#type == MAIL_TYPE_PERSONAL
                    && !mail.lock.unwrap_or(false)
            })
        });
        if self.mails.len() != before {
            self.dirty = true;
        }

        let mail_show = self.mails.iter().map(mail_show_from_base).collect();
        Ok(DelMailRs { mail_show }.encode_to_vec())
    }

    fn cmd_lock_or_not_personal_mail(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let rq = LockOrNotPersonalMailRq::decode(payload)?;
        let mut changed = false;
        let mail_pb = self.personal_mail_mut(rq.key_id).map(|mail| {
            if mail.lock != Some(rq.lock) {
                mail.lock = Some(rq.lock);
                changed = true;
            }
            mail.clone()
        });
        if changed {
            self.dirty = true;
        }
        Ok(LockOrNotPersonalMailRs {
            mail_pb,
            lock: rq.lock,
        }
        .encode_to_vec())
    }

    pub fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        _config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> Result<Vec<u8>> {
        match cmd {
            6001 => self.cmd_get_mail_list(),
            6003 => self.cmd_get_personal_mail_by_id(payload),
            6011 => self.cmd_read_all_mail(payload),
            6013 => self.cmd_del_mail(payload),
            6017 => self.cmd_lock_or_not_personal_mail(payload),
            _ => Err(anyhow::anyhow!("Unknown mail cmd: {}", cmd)),
        }
    }

    fn to_proto(&self) -> MailFunction {
        MailFunction {
            base_mail: self.mails.clone(),
        }
    }
}

fn initial_status(mail: &BaseMailPb) -> i32 {
    if mail.award.is_empty() {
        MAIL_STATUS_UNREAD
    } else {
        MAIL_STATUS_UNREAD_WITH_AWARD
    }
}

fn mark_mail_read(mail: &mut BaseMailPb) -> bool {
    let next = match mail.status.unwrap_or_else(|| initial_status(mail)) {
        MAIL_STATUS_UNREAD => MAIL_STATUS_READ,
        MAIL_STATUS_UNREAD_WITH_AWARD => MAIL_STATUS_READ_WITH_AWARD,
        other => other,
    };
    if mail.status == Some(next) {
        return false;
    }
    mail.status = Some(next);
    true
}

fn mail_show_from_base(mail: &BaseMailPb) -> MailShowPb {
    MailShowPb {
        key_id: mail.key_id.unwrap_or_default(),
        template_id: mail.template_id,
        r#type: mail.r#type,
        time: mail.time,
        state: mail.status,
        lock: mail.lock,
        title: mail.title.clone(),
        content: mail.content.clone(),
        t_param: mail.t_param.clone(),
        c_param: mail.c_param.clone(),
        expired_time: mail.expired_time,
    }
}

impl PlayerSystem for MailSystem {
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()> {
        let func = MailFunction::decode(data)?;
        self.mails = func.base_mail;
        info!(mails = self.mails.len(), "MailSystem loaded");
        Ok(())
    }

    fn save_to_bin(&self) -> Result<Vec<u8>> {
        Ok(self.to_proto().encode_to_vec())
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn column_name(&self) -> &'static str {
        col::MAIL
    }

    fn handle_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &std::sync::Arc<shared::static_config::StaticConfig>,
    ) -> Result<Vec<u8>> {
        MailSystem::handle_command(self, cmd, payload, config)
    }
}

impl shared::msg::ToFunctionClientBaseBytes for MailSystem {
    fn to_function_base_bytes(&self) -> Vec<u8> {
        use shared::msg::{func_tag, func_type};
        shared::msg::build_function_base_bytes_pub(
            func_type::MAIL,
            func_tag::MAIL,
            &self.to_proto(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::{AwardPb, GetPersonalMailByIdRs, ReadAllMailRs};
    use shared::static_config::StaticConfig;
    use std::sync::Arc;

    fn mail(key_id: i32, status: i32, locked: bool) -> BaseMailPb {
        BaseMailPb {
            key_id: Some(key_id),
            template_id: 100,
            r#type: MAIL_TYPE_PERSONAL,
            time: Some(i64::from(key_id)),
            title: Some(format!("mail-{key_id}")),
            content: Some("content".to_string()),
            status: Some(status),
            lock: Some(locked),
            ..Default::default()
        }
    }

    #[test]
    fn add_personal_mail_assigns_id_and_marks_dirty() {
        let mut system = MailSystem::new();

        let key_id = system.add_personal_mail(BaseMailPb {
            template_id: 101,
            r#type: 99,
            award: vec![AwardPb {
                r#type: 4,
                id: 1001,
                count: 1,
                ..Default::default()
            }],
            ..Default::default()
        });

        assert_eq!(key_id, 1);
        assert_eq!(system.mails[0].r#type, MAIL_TYPE_PERSONAL);
        assert_eq!(system.mails[0].status, Some(MAIL_STATUS_UNREAD_WITH_AWARD));
        assert!(system.is_dirty());
    }

    #[test]
    fn get_personal_mail_marks_unread_mail_read() {
        let mut system = MailSystem::new();
        system.mails.push(mail(7, MAIL_STATUS_UNREAD, false));
        let config = Arc::new(StaticConfig::default());

        let resp = system
            .handle_command(
                6003,
                &GetPersonalMailByIdRq { key_id: 7 }.encode_to_vec(),
                &config,
            )
            .unwrap();

        let rs = GetPersonalMailByIdRs::decode(resp.as_slice()).unwrap();
        assert_eq!(rs.mail_pb.unwrap().status, Some(MAIL_STATUS_READ));
        assert!(system.is_dirty());
    }

    #[test]
    fn read_all_mail_only_updates_personal_targets() {
        let mut system = MailSystem::new();
        system.mails.push(mail(1, MAIL_STATUS_UNREAD, false));
        system.mails.push(mail(2, MAIL_STATUS_READ, false));
        let config = Arc::new(StaticConfig::default());

        let resp = system
            .handle_command(
                6011,
                &ReadAllMailRq {
                    key_id: vec![
                        TwoInt {
                            v1: 1,
                            v2: MAIL_TYPE_PERSONAL,
                        },
                        TwoInt { v1: 2, v2: 1 },
                    ],
                }
                .encode_to_vec(),
                &config,
            )
            .unwrap();

        let rs = ReadAllMailRs::decode(resp.as_slice()).unwrap();
        assert_eq!(rs.personal_mail.len(), 1);
        assert_eq!(rs.personal_mail[0].v2, MAIL_STATUS_READ);
        assert_eq!(system.mails[0].status, Some(MAIL_STATUS_READ));
        assert!(system.is_dirty());
    }

    #[test]
    fn del_mail_keeps_locked_personal_mail() {
        let mut system = MailSystem::new();
        system.mails.push(mail(1, MAIL_STATUS_READ, true));
        system.mails.push(mail(2, MAIL_STATUS_READ, false));
        let config = Arc::new(StaticConfig::default());

        system
            .handle_command(
                6013,
                &DelMailRq {
                    key_id: vec![
                        TwoInt {
                            v1: 1,
                            v2: MAIL_TYPE_PERSONAL,
                        },
                        TwoInt {
                            v1: 2,
                            v2: MAIL_TYPE_PERSONAL,
                        },
                    ],
                }
                .encode_to_vec(),
                &config,
            )
            .unwrap();

        assert_eq!(system.mails.len(), 1);
        assert_eq!(system.mails[0].key_id, Some(1));
        assert!(system.is_dirty());
    }

    #[test]
    fn save_round_trip_clears_dirty_only_when_requested() {
        let mut system = MailSystem::new();
        system.add_personal_mail(mail(3, MAIL_STATUS_UNREAD, false));

        let data = system.save_to_bin().unwrap();
        assert!(system.is_dirty());
        system.clear_dirty();
        assert!(!system.is_dirty());

        let mut loaded = MailSystem::new();
        loaded.load_from_bin(&data).unwrap();
        assert_eq!(loaded.mails.len(), 1);
        assert_eq!(loaded.mails[0].key_id, Some(3));
        assert!(!loaded.is_dirty());
    }
}
