package web

import "net/http"

type HandleFunc func(ctx *Context)

type Server interface {
	http.Handler
	// Start 启动服务器
	// addr 是监听地址。如果只指定端口，可以使用 ":8081"
	// 或者 "localhost:8082"
	Start(addr string) error

	// addRoute 注册一个路由
	// method 是 HTTP 方法
	addRoute(method string, path string, handler HandleFunc)
	// 我们并不采取这种设计方案
	// addRoute(method string, path string, handlers... HandleFunc)
}

func (s *HTTPServer) UseAny(path string, mdls ...Middleware) {
	s.addRoute(http.MethodGet, path, nil, mdls...)
	s.addRoute(http.MethodPost, path, nil, mdls...)
	s.addRoute(http.MethodOptions, path, nil, mdls...)
	s.addRoute(http.MethodConnect, path, nil, mdls...)
	s.addRoute(http.MethodDelete, path, nil, mdls...)
	s.addRoute(http.MethodHead, path, nil, mdls...)
	s.addRoute(http.MethodPatch, path, nil, mdls...)
	s.addRoute(http.MethodPut, path, nil, mdls...)
	s.addRoute(http.MethodTrace, path, nil, mdls...)
}

// ServeHTTP HTTPServer 处理请求的入口
func (s *HTTPServer) ServeHTTP(writer http.ResponseWriter, request *http.Request) {
	ctx := &Context{
		Req:       request,
		Resp:      writer,
		tplEngine: s.tplEngine,
	}

	// ctx pool.Get()
	// defer func(){
	//     ctx.Reset()
	//     pool.Put(ctx)
	// }
	s.serve(ctx)
}
