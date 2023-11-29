package iface

type HandleStep int

// IFuncRequest function message interface (函数消息接口)
type IFuncRequest interface {
	CallFunc()
}

type IRequest interface {
	GetConnection() IConnection
}
