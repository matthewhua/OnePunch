package iface

// IMsgHandle Abstract layer of message management
type IMsgHandle interface {

	// AddRouter Add specific handling logic for messages, msgID supports int and string types
	// (为消息添加具体的处理逻辑, msgID，支持整型，字符串)
	AddRouter(msgId uint32, router IRouter)
	AddRouterSlices(msgId uint32, handler ...RouterHandler) IRouterSlices
	Group(start, end uint32, Handlers ...RouterHandler) IGroupRouterSlices
	Use(Handlers ...RouterHandler) IRouterSlices

	StartWorkerPool() //  Start the worker pool

	SendMsgToTaskQueue(request IRequest) // Pass the message to the TaskQueue for processing by the worker(将消息交给TaskQueue,由worker进行处理)

	// Execute interceptor methods on the responsibility chain(执行责任链上的拦截器方法)
	Execute(request IRequest)

	// AddInterceptor Register the entry point of the responsibility chain. After each interceptor is processed,
	// the data is passed to the next interceptor, so that the message can be handled and passed layer by layer,
	// the order depends on the registration order
	// (注册责任链任务入口，每个拦截器处理完后，数据都会传递至下一个拦截器，使得消息可以层层处理层层传递，顺序取决于注册顺序)
	AddInterceptor(interceptor IInterceptor)
}
