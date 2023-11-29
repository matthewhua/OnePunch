package net

type HandlerFunc func()

type WsServer struct {
	Server
}

func NewServer(addr string) *WsServer {
	return &WsServer{
		Server{
			Name: addr,
		},
	}
}

func (s *Server) SetRouter(router *Router) {
	s.router = router
}

type Router struct {
	group []*group
}
