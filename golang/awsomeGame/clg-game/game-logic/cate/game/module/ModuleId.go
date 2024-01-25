package module

const (
	// LOGIN ModuleId (module main message number)
	// 登录
	LOGIN = 1
	// ROLE 玩家
	ROLE = 2
	// BAG 背包
	BAG = 3
	// FIGHT 战斗
	FIGHT = 4
	// HERO 英雄
	HERO = 5
	// MAIL 邮件
	MAIL = 6
	// ADVENTURE 冒险
	ADVENTURE = 7
	// MALL 商城
	MALL = 8
	// HERO_CALL 英雄召唤
	HERO_CALL = 9
	// FRIEND 好友
	FRIEND = 10
	// CLIENT_PAGE 玩家弹幕推送与否【玩家是否在界面中】
	CLIENT_PAGE = 17
	// GUILD 公会
	GUILD = 18
	// GOLDEN_TOUCH 点金
	GOLDEN_TOUCH = 19
	// EMPLOY 雇佣兵
	EMPLOY = 20
	// RD_CENTER 科技树
	RD_CENTER = 23
	// PUSH_GIFT 推送礼包
	PUSH_GIFT = 25
	// SHARE 分享
	SHARE = 29
	// MILITARY_RANK 军事排名
	MILITARY_RANK = 39
	// RELIC 圣物
	RELIC = 47
	// GUILD_BOSS 公会BOSS
	GUILD_BOSS = 85
	// TOTE PLE 图腾圣殿
	TOTEPLE = 86
	// TOWER 玲珑塔
	TOWER = 87
	// GUILD_LEADER 公会首领
	GUILD_LEADER = 92
	// PEEK 巅峰挑战
	PEEK = 94
	// RANK 排行榜
	RANK = 97
	// BLOG 个人空间
	BLOG = 98
	// HISTORY 主角历史
	HISTORY = 99
	// ACTIVITY 活动 (具体活动会加n)
	ACTIVITY = 100
	// TASK 任务
	TASK = 100
	// VIDEO_HALL 录像馆
	VIDEO_HALL = 110
	// TITAN 泰坦
	TITAN = 301
	// GIFT_PACK 礼包码
	GIFT_PACK = 400
	// CHAT 聊天
	CHAT = 666
	// LADDER 天梯
	LADDER = 700
	// GUIDE 新手引导
	GUIDE = 5000
	// COMMON 通用模块
	COMMON = 10000
	// PAYMENT 支付
	PAYMENT = 75000
	// ADMIN 开发指令
	// TODO: 2023/8/10 管理和GM模块作为独立在 module 之外的辅助模块，通过配置直接驱动线上和开发环境
	ADMIN = 99999
)
