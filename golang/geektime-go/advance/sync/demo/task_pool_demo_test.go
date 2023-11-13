package demo

import (
	"fmt"
	"testing"
	"time"
)

func TestTaskPool_Do(t *testing.T) {
	tp := NewTaskPool(2)
	tp.Do(func() {
		time.Sleep(time.Second)
		fmt.Println("task1")
	})

	tp.Do(func() {
		time.Sleep(time.Second)
		fmt.Println("task2")
	})

	tp.Do(func() {
		MyTask(1, "13")
	})
}

func MyTask(a int, b string) {
	//
}
