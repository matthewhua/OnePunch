package net

import (
	"awsomeGame/conf"
	"awsomeGame/iface"
	"awsomeGame/net/decoder"
	"awsomeGame/net/pack"
	"github.com/aceld/zinx/logo"
	"github.com/aceld/zinx/zlog"
	"github.com/gorilla/websocket"
	"net/http"
)

// Server interface implementation, defines a Server service class
// (接口实现，定义一个Server服务类)
type Server struct {
	// Name of the server (服务器的名称)
	Name string
	// tcp4 or other
	IPVersion string
	// IP version (e.g. "tcp4") - 服务绑定的IP地址
	IP string
	// IP address the server is bound to (服务绑定的端口)
	Port int

	// Current server's message handler module, used to bind MsgID to corresponding processing methods
	// (当前Server的消息管理模块，用来绑定MsgID和对应的处理方法)
	msgHandler iface.IMsgHandle

	// Routing mode (路由模式)
	RouterSlicesMode bool

	// Current server's connection manager (当前Server的链接管理器)
	ConnMgr iface.IConnManager

	// Hook function called when a new connection is established
	// (该Server的连接创建时Hook函数)
	onConnStart func(conn iface.IConnection)

	// Hook function called when a connection is terminated
	// (该Server的连接断开时的Hook函数)
	onConnStop func(conn iface.IConnection)

	// Data packet encapsulation method
	// (数据报文封包方式)
	packet iface.IDataPack

	// Asynchronous capture of connection closing status
	// (异步捕获链接关闭状态)
	exitChan chan struct{}

	// Decoder for dealing with message fragmentation and reassembly
	// (断粘包解码器)
	decoder iface.IDecoder

	// Heartbeat checker
	// (心跳检测器)
	hc iface.IHeartbeatChecker

	// websocket
	upgrader *websocket.Upgrader

	// websocket connection authentication
	websocketAuth func(r *http.Request) error

	// connection id
	cID uint64
}

// newServerWithConfig creates a server handle based on config
// (根据config创建一个服务器句柄)
func newServerWithConfig(config *conf.Config, ipVersion string, opts ...Option) iface.IServer {
	logo.PrintLogo()

	s := &Server{
		Name:             config.Name,
		IPVersion:        ipVersion,
		IP:               config.Host,
		Port:             config.TCPPort,
		msgHandler:       newMsgHandle(),
		RouterSlicesMode: config.RouterSlicesMode,
		ConnMgr:          newConnManager(),
		exitChan:         nil,
		// Default to using Zinx's TLV data pack format
		// (默认使用zinx的TLV封包方式)
		packet:  pack.Factory().NewPack(iface.ZinxDataPack),
		decoder: decoder.NewTLVDecoder(), // Default to using TLV decode (默认使用TLV的解码方式)
		upgrader: &websocket.Upgrader{
			ReadBufferSize: int(config.IOReadBuffSize),
			CheckOrigin: func(r *http.Request) bool {
				return true
			},
		},
	}

	for _, opt := range opts {
		opt(s)
	}

	// Display current configuration information
	// (提示当前配置信息)
	config.Show()

	return s
}

// NewServer creates a server handle
// (创建一个服务器句柄)
func NewServer(opts ...Option) iface.IServer {
	return newServerWithConfig(conf.GlobalObject, "tcp", opts...)
}

// NewUserConfServer creates a server handle using user-defined configuration
// (创建一个服务器句柄)
func NewUserConfServer(config *conf.Config, opts ...Option) iface.IServer {
	// Refresh user configuration to global configuration variable
	// (刷新用户配置到全局配置变量)

	conf.UserConfToGlobal(config)
	s := newServerWithConfig(config, "tcp4", opts...)
	return s
}

// NewDefaultRouterSlicesServer creates a server handle with a default RouterRecovery processor.
// (创建一个默认自带一个Recover处理器的服务器句柄)
func NewDefaultRouterSlicesServer(opts ...Option) iface.IServer {
	conf.GlobalObject.RouterSlicesMode = true
	s := newServerWithConfig(conf.GlobalObject, "tcp", opts...)
	s.Use(RouterRecovery)
	return s
}

func (s *Server) AddRouter(msgID uint32, router iface.IRouter) {
	if s.RouterSlicesMode {
		panic("Server RouterSlicesMode is true ")
	}
	s.msgHandler.AddRouter(msgID, router)
}

// Stop stops the server (停止服务)
func (s *Server) Stop() {
	zlog.Ins().InfoF("[STOP] Zinx server , name %s", s.Name)

	// Clear other connection information or other information that needs to be cleaned up
	// (将其他需要清理的连接信息或者其他信息 也要一并停止或者清理)
	s.ConnMgr.ClearConn()
	s.exitChan <- struct{}{}
	close(s.exitChan)
}
