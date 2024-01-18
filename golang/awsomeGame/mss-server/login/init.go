package login

import (
	"awsomeGame/db"
	"awsomeGame/mss-server/login/cmd"
	"net"
)

var Router = net.NewRouter()

func Init() {
	//测试数据库，并且初始化数据库
	db.TestDB()
	//还有别的初始化方法
	initRouter()
}

func initRouter() {
	cmd.Account.Router(Router)
}
