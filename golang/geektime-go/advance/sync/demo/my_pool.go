package demo

import (
	"sync"
	"unsafe"
)

type MyPool struct {
	p      sync.Pool
	maxCnt int32
	cnt    int32
}

func (p *MyPool) Get() any {
	return p.p.Get()
}

func (p *MyPool) Put(v any) {
	// 大对象
	if unsafe.Sizeof(v) > 1024 {
		return
	}
	if v != nil {
		p.cnt++
		if p.cnt > p.maxCnt {
			p.cnt = p.maxCnt
		}
	}
	p.p.Put(v)
}

func (p *MyPool) MaxCnt() int32 {
	return p.maxCnt
}

func (p *MyPool) Cnt() int32 {
	return p.cnt
}
