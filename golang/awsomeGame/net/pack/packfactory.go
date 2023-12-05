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

// NewPack creates a concrete packaging and unpackaging object
// (NewPack 创建一个具体的拆包解包对象)
func (f *PACK_FACTORY) NewPack(kind string) iface.IDataPack {
	var dataPack iface.IDataPack

	switch kind {
	// Zinx standard default packaging and unpackaging method
	// (Zinx 标准默认封包拆包方式)
	case iface.ZinxDataPack:
		dataPack = NewDataPack()
	case iface.ZinxDataPackOld:
		dataPack = NewDataPackLtv()
		// case for custom packaging and unpackaging methods
		// (case 自定义封包拆包方式case)
	default:
		dataPack = NewDataPack()
	}

	return dataPack
}
