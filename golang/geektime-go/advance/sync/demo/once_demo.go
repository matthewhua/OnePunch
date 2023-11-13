package demo

import (
	"fmt"
	"sync"
)

type OnceClose struct {
	close sync.Once
}

func (o *OnceClose) Close() error {
	o.close.Do(func() {
		fmt.Println("I' m closing....")
	})
	return nil
}

func init() {
	// 在这里的动作，肯定执行一次
}
