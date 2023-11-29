package iface

// IMsgHandle Abstract layer of message management
type IMsgHandle interface {

	// AddRouter Add specific handling logic for messages, msgID supports int and string types
	// (为消息添加具体的处理逻辑, msgID，支持整型，字符串)
	AddRouter(msgId uint32, router IRouter)
	AddRouterSlices(msgId uint32, handler ...RouterHandler) IRouterSlices
}
