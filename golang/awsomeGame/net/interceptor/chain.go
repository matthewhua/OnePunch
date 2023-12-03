package interceptor

import (
	"awsomeGame/iface"
)

type Chain struct {
	req          iface.IcReq
	position     int
	interceptors []iface.IInterceptor
}

func NewChain(list []iface.IInterceptor, pos int, req iface.IcReq) iface.IChain {
	return &Chain{
		req:          req,
		position:     pos,
		interceptors: list,
	}
}

func (c *Chain) Request() iface.IcReq {
	return c.req
}

func (c *Chain) Proceed(req iface.IcReq) iface.IcResp {
	if c.position < len(c.interceptors) {
		chain := NewChain(c.interceptors, c.position+1, req)
		interceptor := c.interceptors[c.position]
		response := interceptor.Intercept(chain)
		return response
	}
	return req
}

// GetIMessage  从Chain中获取IMessage
func (c *Chain) GetIMessage() iface.IMessage {
	req := c.Request()
	if req == nil {
		return nil
	}

	iRequest := c.ShouldIRequest(req)
	if iRequest == nil {
		return nil
	}

	return iRequest.GetMessage()
}

// ProceedWithMessage Next 通过IMessage和解码后数据进入下一个责任链任务
// iMessage 为解码后的IMessage
// response 为解码后的数据
func (c *Chain) ProceedWithMessage(iMessage iface.IMessage, response iface.IcReq) iface.IcResp {
	if iMessage == nil || response == nil {
		return c.Proceed(c.Request())
	}

	req := c.Request()
	if req == nil {
		return c.Proceed(c.Request())
	}

	iRequest := c.ShouldIRequest(req)
	if iRequest == nil {
		return c.Proceed(c.Request())
	}

	// 设置chain的request下一次请求
	iRequest.SetResponse(response)
	return c.Proceed(iRequest)
}

// ShouldIRequest 判断是否是IRequest
func (c *Chain) ShouldIRequest(icReq iface.IcReq) iface.IRequest {
	if icReq == nil {
		return nil
	}

	switch icReq.(type) {
	case iface.IRequest:
		return icReq.(iface.IRequest)
	default:
		return nil
	}
}
