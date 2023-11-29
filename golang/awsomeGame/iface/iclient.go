package iface

type IClient interface {
	Restart()
	Start()
	Stop()
	AddRouter(msgId uint32, router IRouter)
	Conn() IConnection

	// SetOnConnStart Set the Hook function to be called when a connection is created for this Client
	// (设置该Client的连接创建时Hook函数)
	SetOnConnStart(func(IConnection))

	// SetOnConnStop Set the Hook function to be called when a connection is closed for this Client
	// (设置该Client的连接断开时的Hook函数)
	SetOnConnStop(func(IConnection))

	// GetOnConnStart Get the Hook function that is called when a connection is created for this Client
	// (获取该Client的连接创建时Hook函数)
	GetOnConnStart() func(IConnection)

	// GetOnConnStop Get the Hook function that is called when a connection is closed for this Client
	// (获取该Client的连接断开时的Hook函数)
	GetOnConnStop() func(IConnection)

	SetPacket(IDataPack)
}
