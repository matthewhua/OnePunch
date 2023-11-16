package demon2

import "net/http"

type Context struct {
	Req    *http.Request
	Writer http.ResponseWriter
	Params map[string]string
}

type HandleFunc func(ctx *Context)

type Server interface {
	http.Handler
	Start(addr string) error
	// AddRoute 注册路由的核心抽象
	AddRoute(method, path string, handler HandleFunc)

	// 不知道怎么调度 handlers
	// 用户一个都不传
	// AddRoutes(method, path string, handlers ...HandleFunc)
}

type HTTPServer struct {
	router
}
