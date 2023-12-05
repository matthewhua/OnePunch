package iface

/*
IDataPack Package and unpack data.
Operating on the data stream of TCP connections, add header information to transfer data, and solve TCP sticky packets.
(封包数据和拆包数据
直接面向TCP连接中的数据流,为传输数据添加头部信息，用于处理TCP粘包问题。)
*/
type IDataPack interface {
	GetHeadLen() uint32
	Pack(msg IMessage) ([]byte, error)
	Unpack([]byte) (IMessage, error)
}

const (
	// Zinx standard packing and unpacking method (Zinx 标准封包和拆包方式)
	ZinxDataPack    string = "zinx_pack_tlv_big_endian"
	ZinxDataPackOld string = "zinx_pack_ltv_little_endian"

	//...(+)
	//// Custom packing method can be added here(自定义封包方式在此添加)
)
