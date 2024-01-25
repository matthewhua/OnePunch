package event

import (
	"sync"
)

type Bus struct {
	listeners map[interface{}][]Runnable
	mutex     sync.Mutex
}

type Runnable func(...interface{})

func NewEventBus() *Bus {
	return &Bus{
		listeners: make(map[interface{}][]Runnable),
	}
}

func (eb *Bus) Register(eventName interface{}, listener Runnable) {
	eb.mutex.Lock()
	defer eb.mutex.Unlock()

	eb.listeners[eventName] = append(eb.listeners[eventName], listener)
}

func (eb *Bus) Trigger(eventName interface{}, args ...interface{}) {
	eb.mutex.Lock()
	defer eb.mutex.Unlock()

	listeners, ok := eb.listeners[eventName]
	if ok {
		for _, listener := range listeners {
			listener(args...)
		}
	}
}
