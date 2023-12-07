package cmd

import (
	"awsomeGame/iface"
	"awsomeGame/mss-server/constant"
	"awsomeGame/net"
)

type Account struct {
	net.BaseRouter
}

func (a *Account) Router(r *net.GroupRouter) {
	r.AddHandler(constant.Login)
}

func (a *Account) login(req *iface.IRequest) {

}
