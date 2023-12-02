package iface

type IDecoder interface {
	IInterceptor
	GetLengthField() *LengthField
}
