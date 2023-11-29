package iface

type IData interface {
	GetHeadLen() uint32
	Pack(msg IM)
}
