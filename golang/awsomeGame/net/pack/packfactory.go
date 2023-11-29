package pack

import (
	"awsomeGame/iface"
	"sync"
)

var pack_once sync.Once

type pack_factory struct{}

var factoryInstance *pack_factory

/*
Factory	Generates different packaging and unpackaging methods, singleton

	(生成不同封包解包的方式，单例)
*/
func Factory() *pack_factory {
	pack_once.Do(func() {
		factoryInstance = new(pack_factory)
	})
	return factoryInstance
}

func (f *pack_factory) NewPack() iface.IDataPack {
	return NewDataPack()
}
