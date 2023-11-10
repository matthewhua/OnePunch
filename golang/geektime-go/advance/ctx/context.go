package ctx

import (
	"context"
	"sync"
)

type Cache interface {
	Get(key string) (string, error)
}

type OtherCache interface {
	GetValue(ctx context.Context, key string) (string, error)
}

// CacheAdapter 适配器强调的是不同接口之间进行适配
// 装饰器强调的是添加额外的功能
type CacheAdapter struct {
	Cache
}

// GetValue 获得Value
func (c *CacheAdapter) GetValue(ctx context.Context, key string) (any, error) {
	return c.Cache.Get(key)
}

// already had, not thread safe
type memoryMap struct {

	// 如果你这样添加锁，那么就是一种侵入式的写法，
	// 那么你就需要测试这个类
	// 而且有些时候，这个是第三方的依赖，你都改不了
	// lock sync.RWMutex
	m map[string]string
}

func (m *memoryMap) Get(key string) (string, error) {
	return m.m[key], nil
}

var safe = &SafeCache{
	Cache: &memoryMap{},
}

// SafeCache 我要改造为线程安全的
// 无侵入式地改造
type SafeCache struct {
	Cache
	lock sync.RWMutex
}

func (s *SafeCache) Get(key string) (string, error) {
	s.lock.RLock()
	defer s.lock.RUnlock()
	return s.Cache.Get(key)
}

type A struct {
	ctx context.Context
}
