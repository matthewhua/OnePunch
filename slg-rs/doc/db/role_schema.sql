-- 领主基础数据表（对应 imperial_sim_game_hqy.p_lord）
-- 存储领主的核心资源和状态，直接按列读写（不走 blob 序列化）
CREATE TABLE IF NOT EXISTS `p_lord` (
    `role_id`                 BIGINT          NOT NULL                COMMENT '角色ID',
    `nick`                    VARCHAR(255)                            COMMENT '昵称',
    `portrait`                VARCHAR(255)                            COMMENT '头像',
    `portrait_frame`          INT                                     COMMENT '头像框',
    `top_up`                  DECIMAL(11,2)                           COMMENT '累计充值金额',
    `diamond`                 BIGINT                                  COMMENT '钻石（付费货币）',
    `diamond_cost`            BIGINT                                  COMMENT '累计消耗钻石',
    `guide_id`                INT                                     COMMENT '当前引导ID',
    `on_time`                 INT                                     COMMENT '创角时间戳',
    `ol_time`                 INT                                     COMMENT '累计在线时长（秒）',
    `off_time`                INT                                     COMMENT '最后下线时间戳',
    `ol_month`                INT                                     COMMENT '本月在线时长',
    `title`                   INT                                     COMMENT '称号ID',
    `max_key`                 INT                                     COMMENT '最大钥匙数',
    `role_status`             VARCHAR(255)                            COMMENT '角色状态JSON',
    `across_day_deal_time`    INT                                     COMMENT '跨天处理时间戳',
    `battle_fight`            BIGINT                                  COMMENT '战斗力',
    `meat`                    BIGINT                                  COMMENT '肉（资源）',
    `fame`                    INT                                     COMMENT '声望',
    `gold`                    BIGINT                                  COMMENT '金币（免费货币）',
    `search_survivor_time`    INT             NOT NULL DEFAULT 0      COMMENT '搜索幸存者时间',
    `stamina`                 BIGINT                                  COMMENT '体力',
    `start_ad_time`           INT                                     COMMENT '广告开始时间',
    `start_ad_id`             INT                                     COMMENT '广告ID',
    `is_add_login`            INT                                     COMMENT '是否已加登录奖励',
    `total_login`             INT                                     COMMENT '累计登录次数',
    `current_streak`          INT                                     COMMENT '连续登录天数',
    `vip_level`               INT                                     COMMENT 'VIP等级',
    `vip_exp`                 INT                                     COMMENT 'VIP经验',
    `camp_id`                 INT                                     COMMENT '阵营ID',
    `last_periodic_task_time` VARCHAR(255)                            COMMENT '上次周期任务时间',
    `lord_system_setting`     VARCHAR(255)                            COMMENT '系统设置JSON',
    `pay_amount`              INT                                     COMMENT '充值总额（分）',
    `language`                VARCHAR(255)                            COMMENT '语言设置',
    `push_switch`             VARCHAR(255)                            COMMENT '推送开关JSON',
    PRIMARY KEY (`role_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='领主基础数据';

-- 玩家功能模块数据表（对应 imperial_sim_game_hqy.p_data）
-- 宽列设计：每个功能模块对应一个 blob 列，存储 protobuf 序列化数据
-- 列名与 Java 版 FunctionEntity 字段名完全一致
CREATE TABLE IF NOT EXISTS `p_data` (
    `role_id`           BIGINT      NOT NULL    COMMENT '角色ID',
    `hero_func`         BLOB                    COMMENT '将领系统',
    `sim_func`          BLOB                    COMMENT '模拟经营系统',
    `backpack_func`     BLOB                    COMMENT '背包系统',
    `technology_func`   BLOB                    COMMENT '科技系统',
    `combat_func`       BLOB                    COMMENT '副本战斗系统',
    `equip_func`        BLOB                    COMMENT '装备系统',
    `world_func`        BLOB                    COMMENT '世界数据',
    `pay_func`          BLOB                    COMMENT '充值系统',
    `mail_func`         BLOB                    COMMENT '邮件系统',
    `guise_func`        BLOB                    COMMENT '外观系统',
    `intel_broker_func` BLOB                    COMMENT '情报商会',
    `camp_func`         BLOB                    COMMENT '阵营系统',
    `activity_func`     BLOB                    COMMENT '活动系统',
    `vip_func`          BLOB                    COMMENT 'VIP系统',
    `wall_func`         BLOB                    COMMENT '城墙系统',
    `shop_func`         BLOB                    COMMENT '商店系统',
    `lord_talent_func`  BLOB                    COMMENT '领主天赋',
    `mission_func`      BLOB                    COMMENT '任务系统',
    `game_play_func`    BLOB                    COMMENT '玩法系统',
    `arena_func`        BLOB                    COMMENT '竞技场',
    `lord_equip_func`   BLOB                    COMMENT '领主装备',
    `skin_func`         BLOB                    COMMENT '皮肤系统',
    `chat_func`         BLOB                    COMMENT '聊天系统',
    `social_func`       BLOB                    COMMENT '社交系统',
    `milestone_func`    BLOB                    COMMENT '里程碑系统',
    PRIMARY KEY (`role_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='玩家功能模块数据';

-- 全服共享数据表（对应 imperial_sim_game_hqy.p_global）
-- 按 server_id 分区，每个区服一行，多个具名 blob 列
CREATE TABLE IF NOT EXISTS `p_global` (
    `server_id`             INT             NOT NULL    COMMENT '区服号',
    `common_mail`           MEDIUMBLOB                  COMMENT '全服邮件',
    `map_data`              MEDIUMBLOB                  COMMENT '世界地图数据',
    `camp_data`             BLOB                        COMMENT '阵营数据',
    `activity_global`       BLOB                        COMMENT '活动全局数据',
    `rank_data`             BLOB                        COMMENT '排行榜数据',
    `chat_data`             BLOB                        COMMENT '聊天数据',
    `gameplay_global_data`  BLOB                        COMMENT '玩法全局数据',
    `milestone_global_data` BLOB                        COMMENT '里程碑全局数据',
    PRIMARY KEY (`server_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='全服共享数据';

-- 服务器配置表（对应 imperial_sim_game_hqy.p_server_config）
CREATE TABLE IF NOT EXISTS `p_server_config` (
    `param_id`      INT             NOT NULL AUTO_INCREMENT,
    `title`         CHAR(20)        NOT NULL    COMMENT '配置标题',
    `param_name`    CHAR(30)        NOT NULL    COMMENT '配置键名',
    `param_value`   VARCHAR(255)    NOT NULL    COMMENT '配置值',
    `descs`         VARCHAR(255)    NOT NULL    COMMENT '说明',
    PRIMARY KEY (`param_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='服务器配置';

-- 初始化服务器配置
INSERT IGNORE INTO `p_server_config` (title, param_name, param_value, descs) VALUES
('账号服地址',  'accountServerUrl', 'http://127.0.0.1:9200/simulate/center/inner.do', '账号服验证URL'),
('区号',        'serverId',         '2001',  '当前区服号'),
('配置方式',    'configMode',       'db',    '服务器配置方式(db/file)');
