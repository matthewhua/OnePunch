package net

import (
	"awsomeGame/iface"
	"awsomeGame/net/interceptor"
)

// chainBuilder is a builder for creating a chain of interceptors.
// (责任链构造器)
type chainBuilder struct {
	body       []iface.IInterceptor
	head, tail iface.IInterceptor
}

// newChainBuilder creates a new instance of chainBuilder.
func newChainBuilder() *chainBuilder {
	return &chainBuilder{
		body: make([]iface.IInterceptor, 0),
	}
}

// Head adds an interceptor to the head of the chain.
func (ic *chainBuilder) Head(interceptor iface.IInterceptor) {
	ic.head = interceptor
}

// Tail adds an interceptor to the tail of the chain.
func (ic *chainBuilder) Tail(interceptor iface.IInterceptor) {
	ic.tail = interceptor
}

// AddInterceptor adds an interceptor to the body of the chain.
func (ic *chainBuilder) AddInterceptor(interceptor iface.IInterceptor) {
	ic.body = append(ic.body, interceptor)
}

// Execute executes all the interceptors in the current chain in order.
func (ic *chainBuilder) Execute(req iface.IcReq) iface.IcResp {

	// Put all the interceptors into the builder
	var interceptors []iface.IInterceptor
	if ic.head != nil {
		interceptors = append(interceptors, ic.head)
	}
	if len(ic.body) > 0 {
		interceptors = append(interceptors, ic.body...)
	}
	if ic.tail != nil {
		interceptors = append(interceptors, ic.tail)
	}

	//  Create a new interceptor chain and execute each interceptor
	chain := interceptor.NewChain(interceptors, 0, req)

	// Execute the chain
	return chain.Proceed(req)
}
