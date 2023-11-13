package unsafe

import (
	"fmt"
	"geektime-go/advance/unsafe/types"
	"testing"
	"unsafe"
)

func TestPrintFiledOffset(t *testing.T) {

	// 目前看来都是64字节， 后面仔细研究一番
	fmt.Println(unsafe.Sizeof(types.User{}))
	PrintFieldOffset(types.User{})

	fmt.Println(unsafe.Sizeof(types.UserV1{}))
	PrintFieldOffset(types.UserV1{})

	fmt.Println(unsafe.Sizeof(types.UserV2{}))
	PrintFieldOffset(types.UserV2{})
}
