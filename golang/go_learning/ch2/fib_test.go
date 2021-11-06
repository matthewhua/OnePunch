package ch2

import "testing"

func TestFibList(t *testing.T) {
	// var a int = 1 //不需要使用了
	// var b int = 1
	// var (
	// 	a int = 1
	// 	b     = 1
	// )
	a := 1

	b := 1
	t.Log(a)
	for i := 0; i < 5; i++ {
		t.Log(" ", b)
		tmp := a
		a = b
		b = tmp + a
	}
}

// go 语言特殊交换
func TestExchange(t *testing.T) {
	a := 1
	b := 2
	// tmp := a
	// a = b
	// b = tmp
	a, b = b, a
	t.Log(a, b)
}
