package iface

import (
	"context"
	"github.com/gorilla/websocket"
	"net"
)

// IConnection // Define connection interface
type IConnection interface {
	// Start the connection, make the current connection start working
	// (启动连接，让当前连接开始工作)
	Start()
	// Stop the connection and end the current connection state
	// (停止连接，结束当前连接状态)
	Stop()

	// Context Returns ctx, used by user-defined go routines to obtain connection exit status
	// (返回ctx，用于用户自定义的go程获取连接退出状态)
	Context() context.Context

	GetName() string            // Get the current connection name (获取当前连接名称)
	GetConnection() net.Conn    // Get the original socket from the current connection(从当前连接获取原始的socket)
	GetWsConn() *websocket.Conn // Get the original websocket connection from the current connection(从当前连接中获取原始的websocket连接)
	GetConnID() uint64          // Get the current connection ID (获取当前连接ID)
	GetConnIdStr() string       // Get the current connection ID for string (获取当前字符串连接ID)

	IsAlive() bool                          // Check if the current connection is alive(判断当前连接是否存活)
	SetHeartBeat(checker IHeartbeatChecker) // Set the heartbeat detector (设置心跳检测器)
}
