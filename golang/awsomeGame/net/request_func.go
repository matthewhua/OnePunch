package net

import (
	"awsomeGame/iface"
)

type RequestFunc struct {
	iface.BaseRequest
	conn     iface.IConnection
	callFunc func()
}

func (rf *RequestFunc) GetConnection() iface.IConnection {
	return rf.conn
}

func (rf *RequestFunc) CallFunc() {
	if rf.callFunc != nil {
		rf.callFunc()
	}
}

func NewFuncRequest(conn iface.IConnection, callFunc func()) iface.IRequest {
	req := new(RequestFunc)
	req.conn = conn
	req.callFunc = callFunc
	return req
}
