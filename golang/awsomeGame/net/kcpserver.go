package net

import (
	"fmt"
	"github.com/xtaci/kcp-go"
)

type KcpServer struct {
}

func (s *KcpServer) ListenKcpConn() {
	// 1. Listen to the server address
	listener, err := kcp.Listen(fmt.Sprintf("%s:%d", s.IP, s.KcpPort))
	if err != nil {
		zlog.Ins().ErrorF("[START] resolve KCP addr err: %v\n", err)
		return
	}
}
