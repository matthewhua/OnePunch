package pack

import (
	"awsomeGame/iface"
	"sync"
)

var PACK_ONCE sync.Once

type PACK_FACTORY struct{}

var factoryInstance *PACK_FACTORY

/*
Factory	Generates different packaging and unPackaging methods, singleton

	(生成不同封包解包的方式，单例)
*/
func Factory() *PACK_FACTORY {
	PACK_ONCE.Do(func() {
		factoryInstance = new(PACK_FACTORY)
	})
	return factoryInstance
}

func (f *PACK_FACTORY) NewPack() iface.IDataPack {
	return NewDataPack()
}
