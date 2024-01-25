package utils

import (
	"sync"
)

type EventBus struct {
	listeners map[string][]Runnable
	mutex     sync.Mutex
}

type Runnable func(...interface{})

func NewEventBus() *EventBus {
	return &EventBus{
		listeners: make(map[string][]Runnable),
	}
}

func (eb *EventBus) Register(eventName string, listener Runnable) {
	eb.mutex.Lock()
	defer eb.mutex.Unlock()

	eb.listeners[eventName] = append(eb.listeners[eventName], listener)
}

func (eb *EventBus) Trigger(eventName string, args ...interface{}) {
	eb.mutex.Lock()
	defer eb.mutex.Unlock()

	listeners, ok := eb.listeners[eventName]
	if ok {
		for _, listener := range listeners {
			listener(args...)
		}
	}
}
