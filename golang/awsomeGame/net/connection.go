package net

import (
	"awsomeGame/conf"
	"awsomeGame/iface"
	"awsomeGame/net/interceptor"
	"context"
	"github.com/aceld/zinx/zlog"
	"net"
	"strconv"
	"sync"
	"time"
)

// Connection TCP connection module
// Used to handle the read and write business of TCP connections, one Connection corresponds to one connection
// (用于处理Tcp连接的读写业务 一个连接对应一个Connection)
type Connection struct {
	// // The socket TCP socket of the current connection(当前连接的socket TCP套接字)
	conn net.Conn

	// The ID of the current connection, also known as SessionID, globally unique, used by server Connection
	// uint64 range: 0~18,446,744,073,709,551,615
	// This is the maximum number of connID theoretically supported by the process
	// (当前连接的ID 也可以称作为SessionID，ID全局唯一 ，服务端Connection使用
	// uint64 取值范围：0 ~ 18,446,744,073,709,551,615
	// 这个是理论支持的进程connID的最大数量)
	connID uint64

	// connection id for string
	// (字符串的连接id)
	connIdStr string

	// The workerId responsible for handling the link
	// 负责处理该链接的workerId
	workerID uint32

	// The message management module that manages MsgID and the corresponding processing method
	// (消息管理MsgID和对应处理方法的消息管理模块)
	msgHandler iface.IMsgHandle

	// Channel to notify that the connection has exited/stopped
	// (告知该链接已经退出/停止的channel)
	ctx    context.Context
	cancel context.CancelFunc

	// Buffered channel used for message communication between the read and write goroutines
	// (有缓冲管道，用于读、写两个goroutine之间的消息通信)
	msgBuffChan chan []byte

	// Lock for user message reception and transmission
	// (用户收发消息的Lock)
	msgLock sync.RWMutex

	// Connection properties
	// (链接属性)
	property map[string]interface{}

	// Lock to protect the current property
	// (保护当前property的锁)
	propertyLock sync.Mutex

	// The current connection's close state
	// (当前连接的关闭状态)
	isClosed bool

	// Which Connection Manager the current connection belongs to
	// (当前链接是属于哪个Connection Manager的)
	connManager iface.IConnManager

	// Hook function when the current connection is created
	// (当前连接创建时Hook函数)
	onConnStart func(conn iface.IConnection)

	// Hook function when the current connection is disconnected
	// (当前连接断开时的Hook函数)
	onConnStop func(conn iface.IConnection)

	// Data packet packaging method
	// (数据报文封包方式)
	packet iface.IDataPack

	// Last activity time
	// (最后一次活动时间)
	lastActivityTime time.Time

	// frameDecoder for solving fragmentation and packet sticking problems
	// (断粘包解码器)
	frameDecoder iface.IFrameDecoder

	// Heartbeat checker
	// (心跳检测室)
	hc iface.IHeartbeatChecker

	// Connection name, default to be the same as the name of the Server/Client that created the connection
	// (链接名称，默认与创建链接的Server/Client的Name一致)
	name string

	// Local address of the current connection
	// (当前链接的本地地址)
	localAddr string

	// Remote address of the current connection
	// (当前链接的远程地址)
	remoteAddr string
}

// newServerConn :for Server, method to create a Server-side connection with Server-specific properties
// (创建一个Server服务端特性的连接的方法)
func newServerConn(server iface.IServer, conn net.Conn, connID uint64) iface.IConnection {

	// Initialize Conn properties
	c := &Connection{
		conn:        conn,
		connID:      connID,
		connIdStr:   strconv.FormatUint(connID, 10),
		isClosed:    false,
		msgBuffChan: nil,
		property:    nil,
		name:        server.ServerName(),
		localAddr:   conn.LocalAddr().String(),
		remoteAddr:  conn.RemoteAddr().String(),
	}

	lengthField := server.GetLengthField()
	if lengthField != nil {
		c.frameDecoder = interceptor.NewFrameDecoder(*lengthField)
	}

	// Inherited properties from server (从server继承过来的属性)
	c.packet = server.GetPacket()
	c.onConnStart = server.GetOnConnStart()
	c.onConnStop = server.GetOnConnStop()
	c.msgHandler = server.GetMsgHandler()

	// Bind the current Connection with the Server's ConnManager
	// (将当前的Connection与Server的ConnManager绑定)
	c.connManager = server.GetConnMgr()

	// Add the newly created Conn to the connection manager
	// (将新创建的Conn添加到链接管理中)
	server.GetConnMgr().Add(c)

	return c
}

func (c *Connection) callOnConnStart() {
	if c.onConnStart != nil {
		zlog.Ins().InfoF("ZINX CallOnConnStart....")
		c.onConnStart(c)
	}
}

func (c *Connection) callOnConnStop() {
	if c.onConnStop != nil {
		zlog.Ins().InfoF("ZINX CallOnConnStop....")
		c.onConnStop(c)
	}
}

func (c *Connection) IsAlive() bool {
	if c.isClosed {
		return false
	}
	// Check the last activity time of the connection. If it's beyond the heartbeat interval,
	// then the connection is considered dead.
	// (检查连接最后一次活动时间，如果超过心跳间隔，则认为连接已经死亡)
	return time.Now().Sub(c.lastActivityTime) < conf.GlobalObject.HeartbeatMaxDuration()
}

func (c *Connection) updateActivity() {
	c.lastActivityTime = time.Now()
}

func (c *Connection) SetHeartBeat(checker iface.IHeartbeatChecker) {
	c.hc = checker
}

func (c *Connection) LocalAddrString() string {
	return c.localAddr
}

func (c *Connection) RemoteAddrString() string {
	return c.remoteAddr
}

func (c *Connection) GetName() string {
	return c.name
}

func (c *Connection) GetMsgHandler() iface.IMsgHandle {
	return c.msgHandler
}
