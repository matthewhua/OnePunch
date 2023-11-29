package demo

import (
	"fmt"
	"net/http"
	"testing"
)

func TestServer(t *testing.T) {
	s := NewHTTPServer()
	s.Get("/", func(ctx *Context) {
		ctx.Resp.Write([]byte("hello World"))
	})

	s.Get("/user", func(ctx *Context) {
		ctx.Resp.Write([]byte("hello, user"))
	})

	s.Get("/user/*", func(ctx *Context) {
		ctx.Resp.Write([]byte("hello, user star"))
	})

	s.Get("/user/home/:id", func(ctx *Context) {
		ctx.Resp.Write([]byte(fmt.Sprintf("hello, user home %s", ctx.Params["id"])))
	})

	g := s.Group("/order")

	g.AddRoute(http.MethodGet, "/detail", func(ctx *Context) {
		ctx.Resp.Write([]byte("hello, order detail"))
	})

	mg := NewGroup(s, "/product")

	mg.AddRoute(http.MethodGet, "/detail", func(ctx *Context) {
		ctx.Resp.Write([]byte("hello, product detail"))
	})
	s.Start(":8081")
}

type MyGroup struct {
	// ms []Middleware
	prefix string
	s      Server
}

func NewGroup(s Server, prefix string) *MyGroup {
	return &MyGroup{prefix, s}
}

func (g *MyGroup) AddRoute(method string, path string, fn func(ctx *Context)) {
	g.s.AddRoute(method, g.prefix+path, fn)
}
