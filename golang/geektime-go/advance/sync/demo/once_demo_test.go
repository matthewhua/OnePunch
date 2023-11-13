package demo

import "testing"

func TestOnceClose_Close(t *testing.T) {
	o := &OnceClose{}
	for i := 0; i < 100; i++ {
		err := o.Close()
		if err != nil {
			return
		}
	}
}
