package demo

// ArrayList 基于切片的简单封装
type ArrayList[T any] struct {
	vals []T
}

func NewArrayList[T any](cap int) *ArrayList[T] {
	panic("implement me")
}

func NewArrayListOf[T any](ts []T) *ArrayList[T] {
	return &ArrayList[T]{
		vals: ts,
	}
}

func (a *ArrayList[T]) Get(index int) (T, error) {
	// TODO implement me
	panic("implement me")
}

func (a *ArrayList[T]) Append(t T) error {
	// TODO implement me
	panic("implement me")
}

// Add 在ArrayList下标为index的位置插入一个元素
// 当index等于ArrayList长度等同于append
func (a *ArrayList[T]) Add(index int, v T) error {
	if index < 0 || index > len(a.vals) {
		return newErrIndexOutOfRange(len(a.vals), index)
	}
	a.vals = append(a.vals, v)
	copy(a.vals[index+1:], a.vals[index:])
	a.vals[index] = v
	return nil
}

func (a *ArrayList[T]) Delete(index int) (T, error) {
	// TODO implement me
	panic("implement me")
}

func (a *ArrayList[T]) Len() int {
	// TODO implement me
	panic("implement me")
}

func (a *ArrayList[T]) Cap() int {
	return cap(a.vals)
}

func (a *ArrayList[T]) Range(fn func(index int, t T) error) error {
	for key, value := range a.vals {
		e := fn(key, value)
		if e != nil {
			return e
		}
	}
	return nil
}

func (a *ArrayList[T]) AsSlice() []T {
	slice := make([]T, len(a.vals))
	copy(slice, a.vals)
	return slice
}
