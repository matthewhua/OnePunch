-- 账号表（对应 imperial_sim_game_hqy.p_account）
-- key_id: 自增主键（内部ID）
-- account_key_id: 账号唯一ID（来自账号服，跨服唯一）
-- role_id: 该账号在本服的角色ID
CREATE TABLE IF NOT EXISTS `p_account` (
    `key_id`          INT         NOT NULL AUTO_INCREMENT COMMENT '内部自增ID',
    `account_key_id`  BIGINT      NOT NULL                COMMENT '账号唯一ID（来自账号服）',
    `server_id`       INT         NOT NULL                COMMENT '区服号',
    `plat_no`         INT         NOT NULL DEFAULT 1      COMMENT '渠道号',
    `publisher`       INT         NOT NULL DEFAULT 1      COMMENT '发行商',
    `plat_id`         CHAR(40)    NOT NULL                COMMENT '渠道账号ID',
    `child_no`        INT         NOT NULL DEFAULT 0      COMMENT '子渠道号',
    `forbid`          INT         NOT NULL DEFAULT 0      COMMENT '封号状态（0正常 1封禁）',
    `white_name`      INT         NOT NULL DEFAULT 0      COMMENT '白名单',
    `role_id`         BIGINT      NOT NULL                COMMENT '角色ID',
    `created`         INT         NOT NULL DEFAULT 0      COMMENT '是否已创建角色（0否 1是）',
    `device_no`       CHAR(80)                            COMMENT '设备号',
    `create_date`     DATETIME                            COMMENT '创建时间',
    `login_days`      INT         NOT NULL DEFAULT 1      COMMENT '累计登录天数',
    `login_date`      DATETIME                            COMMENT '最后登录时间',
    `is_gm`           INT         NOT NULL DEFAULT 0      COMMENT 'GM标记',
    `is_guider`       INT         NOT NULL DEFAULT 0      COMMENT '引导员标记',
    `guidance`        INT                                 COMMENT '引导类型',
    `log_off`         TINYINT                             COMMENT '是否已下线（0在线 1下线）',
    `silence_time`    BIGINT                              COMMENT '禁言到期时间戳',
    `open_id`         VARCHAR(255)                        COMMENT '第三方平台openId',
    `pack_id`         VARCHAR(255)                        COMMENT '包ID',
    PRIMARY KEY (`key_id`),
    UNIQUE KEY `uk_role_id` (`role_id`),
    KEY `idx_account_server` (`account_key_id`, `server_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='账号表';
