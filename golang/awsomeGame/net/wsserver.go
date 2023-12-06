package net

import (
	"awsomeGame/conf"
	"fmt"
	"github.com/aceld/zinx/zconf"
	"github.com/aceld/zinx/zlog"
	"github.com/gorilla/websocket"
	"net/http"
	"sync/atomic"
)

type HandlerFunc func()

type WsServer struct {
	Server
}

func NewWsServer(config *conf.Config) *WsServer {
	return &WsServer{
		Server{},
	}
}

// 监听websocket链接
func (s *WsServer) ListenSocket() {

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		// 1. Check if the server has reached the maximum allowed number of connections
		// (设置服务器最大连接控制,如果超过最大连接，则等待)
		if s.ConnMgr.Len() >= zconf.GlobalObject.MaxConn {
			zlog.Ins().InfoF("Exceeded the maxConnNum:%d, Wait:%d", zconf.GlobalObject.MaxConn, AcceptDelay.duration)
			AcceptDelay.Delay()
			return
		}
		// 2. If websocket authentication is required, set the authentication information
		// (如果需要 websocket 认证请设置认证信息)
		if s.websocketAuth != nil {
			err := s.websocketAuth(r)
			if err != nil {
				zlog.Ins().ErrorF(" websocket auth err:%v", err)
				w.WriteHeader(401)
				AcceptDelay.Delay()
				return
			}
		}
		// 3. Check if there is a subprotocol specified in the header
		// (判断 header 里面是有子协议)
		if len(r.Header.Get("Sec-Websocket-Protocol")) > 0 {
			s.upgrader.Subprotocols = websocket.Subprotocols(r)
		}
		// 4. Upgrade the connection to a websocket connection
		// (升级成 websocket 连接)
		conn, err := s.upgrader.Upgrade(w, r, nil)
		if err != nil {
			zlog.Ins().ErrorF("new websocket err:%v", err)
			w.WriteHeader(500)
			AcceptDelay.Delay()
			return
		}
		AcceptDelay.Reset()
		// 5. Handle the business logic of the new connection, which should already be bound to a handler and conn
		// 5. 处理该新连接请求的 业务 方法， 此时应该有 handler 和 conn是绑定的
		newCid := atomic.AddUint64(&s.cID, 1)
		wsConn := newWebsocketConn(s, conn, newCid)
		go s.StartConn(wsConn)

	})

	err := http.ListenAndServe(fmt.Sprintf("%s:%d", s.IP, s.Port), nil)
	if err != nil {
		panic(err)
	}
}
