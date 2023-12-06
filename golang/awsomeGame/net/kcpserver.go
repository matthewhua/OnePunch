package net

import (
	"fmt"
	"github.com/aceld/zinx/zconf"
	"github.com/aceld/zinx/zlog"
	"github.com/xtaci/kcp-go"
	"sync/atomic"
)

type KcpServer struct {
	Server
}

func (s *KcpServer) ListenSocket() {

	// 1. Listen to the server address
	listener, err := kcp.Listen(fmt.Sprintf("%s:%d", s.IP, s.Port))
	if err != nil {
		zlog.Ins().ErrorF("[START] resolve KCP addr err: %v\n", err)
		return
	}

	zlog.Ins().InfoF("[START] KCP server listening at IP: %s, Port %d, Addr %s", s.IP, s.Port, listener.Addr().String())

	// 2. Start server network connection business
	go func() {
		for {
			// 2.1 Set the maximum connection control for the server. If it exceeds the maximum connection, wait.
			// (设置服务器最大连接控制,如果超过最大连接，则等待)
			if s.ConnMgr.Len() >= zconf.GlobalObject.MaxConn {
				zlog.Ins().InfoF("Exceeded the maxConnNum:%d, Wait:%d", zconf.GlobalObject.MaxConn, AcceptDelay.duration)
				AcceptDelay.Delay()
				continue
			}
			// 2.2 Block and wait for a client to establish a connection request.
			// (阻塞等待客户端建立连接请求)
			conn, err := listener.Accept()
			if err != nil {
				zlog.Ins().ErrorF("Accept KCP err: %v", err)
				AcceptDelay.Delay()
				continue
			}

			AcceptDelay.Reset()

			// 3.4 Handle the business method for this new connection request. At this time, the handler and conn should be bound.
			// (处理该新连接请求的 业务 方法， 此时应该有 handler 和 conn 是绑定的)
			newCid := atomic.AddUint64(&s.cID, 1)
			dealConn := newKcpServerConn(s, conn.(*kcp.UDPSession), newCid)

			go s.StartConn(dealConn)
		}
	}()
	select {
	case <-s.exitChan:
		err := listener.Close()
		if err != nil {
			zlog.Ins().ErrorF("KCP listener close err: %v", err)
		}
	}
}
