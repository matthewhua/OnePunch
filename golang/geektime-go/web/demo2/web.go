package demo

import (
	"net"
	"net/http"
)

type Context struct {
	Req    *http.Request
	Resp   http.ResponseWriter
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

func NewHTTPServer() *HTTPServer {
	return &HTTPServer{
		router: newRouter(),
	}
}

// 实现这个方法
// user/123

// /user/*
// /user/:id
// /user/:name(.+)
func (m *HTTPServer) AddRoute(method, path string, handler HandleFunc) {
	m.addRoute(method, path, handler)
}

func (m *HTTPServer) Get(path string, handler HandleFunc) {
	m.AddRoute(http.MethodGet, path, handler)
}

func (m *HTTPServer) Post(path string, handler HandleFunc) {
	m.AddRoute(http.MethodPost, path, handler)
}

func (m *HTTPServer) serve(ctx *Context) {
	mi, ok := m.findRoute(ctx.Req.Method, ctx.Req.URL.Path)
	// 没找到，404
	if !ok || mi.n.handler == nil {
		ctx.Resp.WriteHeader(http.StatusNotFound)
		ctx.Resp.Write([]byte("404 Not Found"))
		return
	}
	ctx.Params = mi.pathParams
	mi.n.handler(ctx)
}

func (m *HTTPServer) Group(prefix string) *Group {
	return &Group{
		prefix: prefix,
		s:      m,
	}
}

type Group struct {
	// ms []Middleware
	prefix string
	s      Server
}

func (m *Group) AddRoute(method, path string, handler HandleFunc) {
	m.s.AddRoute(method, m.prefix+path, handler)
}

func (m *HTTPServer) Start(addr string) error {
	// 端口启动前
	listener, err := net.Listen("tcp", ":8081")
	if err != nil {
		return err
	}
	// 端口启动后
	// web 服务的服务发现
	// 注册本服务器到你的管理平台
	// 比如说你注册到 etcd，然后你打开管理界面，你就能看到这个实例
	// 10.0.0.1:8081
	println("成功监听端口 8081")
	// http.Serve 接收了一个 Listener
	return http.Serve(listener, m)
	// 这个是阻塞的
	/*return http.ListenAndServe(addr, m)
	// 你没办法在这里做点什么*/
}

type HTTPSServer struct {
	// HTTPServer
	Server
	CertFile string
	KeyFile  string
}

func (m *HTTPSServer) Start(addr string) error {
	return http.ListenAndServeTLS(addr, m.CertFile, m.KeyFile, m)
}

func (m *HTTPServer) ServeHTTP(writer http.ResponseWriter, request *http.Request) {
	ctx := &Context{
		Req:  request,
		Resp: writer,
	}
	// 接下来就是
	// 查找路由
	// 执行业务逻辑
	m.serve(ctx)
}
