package net

import (
	"awsomeGame/iface"
	"strconv"
	"sync"
)

// BaseRouter is used as the base class when implementing a router.
// Depending on the needs, the methods of this base class can be overridden.
// (实现router时，先嵌入这个基类，然后根据需要对这个基类的方法进行重写)
type BaseRouter struct{}

func (br *BaseRouter) PreHandle(req iface.IRequest) {}

func (br *BaseRouter) Handle(req iface.IRequest) {}

func (br *BaseRouter) PostHandle(req iface.IRequest) {}

// New slice-based router
// The new version of the router has basic logic that allows users to pass in varying numbers of router handlers.
// The router will save all of these router handler functions and find them when a request comes in, then execute them using IRequest.
// The router can set globally shared components using the Use method.
// The router can be grouped using Group, and groups also have their own Use method for setting group-shared components.
// (新切片集合式路由
// 新版本路由基本逻辑,用户可以传入不等数量的路由路由处理器
// 路由本体会讲这些路由处理器函数全部保存,在请求来的时候找到，并交由IRequest去执行
// 路由可以设置全局的共用组件通过Use方法
// 路由可以分组,通过Group,分组也有自己对应Use方法设置组共有组件)

type RouterSlices struct {
	Apis     map[uint32][]iface.RouterHandler
	Handlers []iface.RouterHandler
	sync.RWMutex
}

func NewRouterSlices() *RouterSlices {
	return &RouterSlices{
		// fixme 后面加入配置
		Apis:     make(map[uint32][]iface.RouterHandler, 10),
		Handlers: make([]iface.RouterHandler, 0, 6),
	}
}

func (r *RouterSlices) Use(handlers ...iface.RouterHandler) {
	r.Handlers = append(r.Handlers, handlers...)
}

func (r *RouterSlices) AddHandler(MsgId uint32, handlers ...iface.RouterHandler) {
	// 1. Check if the API handler method bound to the current msg already exists
	if _, ok := r.Apis[MsgId]; ok {
		panic("repeated api, msgId = " + strconv.Itoa(int(MsgId)))
	}
}
