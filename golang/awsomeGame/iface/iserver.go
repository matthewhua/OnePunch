package iface

type IServer interface {
	Start() // Start the server method(启动服务器方法)
	Stop()  // Stop the server method (停止服务器方法)
	Serve() // Start the business service method(开启业务服务方法)

	// AddRouter Routing feature: register a routing business method for the current service for client link processing use
	//(路由功能：给当前服务注册一个路由业务方法，供客户端链接处理使用)
	AddRouter(msgID uint32, router IRouter)
}
