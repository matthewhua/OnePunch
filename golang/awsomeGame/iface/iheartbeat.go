package iface

type IHeartbeatChecker interface {
	Start()
	Stop()
	SendHeartBeatMsg()
	BindConn(connection IConnection)
	Clone() IHeartbeatChecker
	MsgId() uint32
	Router() IRouter
	RouterSlices() []RouterHandler
}
