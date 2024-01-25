package event

import (
	"fmt"
	"sync"
	"testing"
)

type MyKey struct {
	Key1 int
	Key2 string
}

type MyStruct struct {
	Field1 int
	Field2 string
}

func myRunnable(args ...interface{}) {
	for _, arg := range args {
		switch v := arg.(type) {
		case MyStruct:
			fmt.Printf("MyStruct: {Field1: %d, Field2: %s}\n", v.Field1, v.Field2)
		default:
			fmt.Println("Unknown type")
		}
	}
}

func TestBus_Trigger(t *testing.T) {

	bus := NewEventBus()

	// Register an event with a struct key
	bus.Register(MyKey{Key1: 1, Key2: "event1"}, myRunnable)

	// Trigger the event with a struct as argument
	bus.Trigger(MyKey{Key1: 1, Key2: "event1"}, MyStruct{Field1: 123, Field2: "hello"})
}

func TestBus_Register(t *testing.T) {
	type fields struct {
		listeners map[interface{}][]Runnable
		mutex     sync.Mutex
	}
	type args struct {
		eventName interface{}
		listener  Runnable
	}
	var tests []struct {
		name   string
		fields fields
		args   args
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			eb := &Bus{
				listeners: tt.fields.listeners,
				mutex:     tt.fields.mutex,
			}
			eb.Register(tt.args.eventName, tt.args.listener)
		})
	}
}

func TestBus_Trigger1(t *testing.T) {
	type fields struct {
		listeners map[interface{}][]Runnable
		mutex     sync.Mutex
	}
	type args struct {
		eventName interface{}
		args      []interface{}
	}
	var tests []struct {
		name   string
		fields fields
		args   args
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			eb := &Bus{
				listeners: tt.fields.listeners,
				mutex:     tt.fields.mutex,
			}
			eb.Trigger(tt.args.eventName, tt.args.args...)
		})
	}
}
