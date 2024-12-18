package sync

import "sync"

type SafeMap[K comparable, V any] struct {
	m     map[K]V
	mutex sync.RWMutex
}

func (s *SafeMap[K, V]) LoadOrStore(key K, newVal V) (val V, loaded bool) {
	oldVal, ok := s.get(key)

}

func (s *SafeMap[K, V]) get(key K) (V, bool) {
	s.mutex.RLock()
	defer s.mutex.RUnlock()
	oldVal, ok := s.m[key]
	return oldVal, ok
}

type ConcurrentArrayList[T any] struct {
	mutex sync.RWMutex
	vals  []T
}

func NewConcurrentArrayList[T any](initCap int) *ConcurrentArrayList[T] {
	return &ConcurrentArrayList[T]{
		vals: make([]T, 0, initCap),
	}
}

func (c *ConcurrentArrayList[T]) Get(index int) T {
	c.mutex.RLock()
	defer c.mutex.RUnlock()
	res := c.vals[index]

	return res
}

func (c *ConcurrentArrayList[T]) DeleteAt(index int) T {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	res := c.vals[index]
	c.vals = append(c.vals[:index], c.vals[index+1:]...)
	return res
}

func (c *ConcurrentArrayList[T]) Append(val T) {

}
