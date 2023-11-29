package net

type HandlerFunc func()

type Server struct {
	addr   string
	router *Router
}

func NewServer(addr string) *Server {
	return &Server{
		addr: addr,
	}
}

func (s *Server) SetRouter(router *Router) {
	s.router = router
}

type Router struct {
	group []*group
}
