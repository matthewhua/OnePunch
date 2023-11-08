package reflect

import (
	"errors"
	"reflect"
)

type ReflectAccessor struct {
	value reflect.Value
	typ   reflect.Type
}

func NewReflectAccessor(val any) (*ReflectAccessor, error) {
	typ := reflect.TypeOf(val)
	if typ.Kind() != reflect.Pointer || typ.Elem().Kind() != reflect.Struct {
		return nil, errors.New("invalid entity")
	}

	return &ReflectAccessor{
		value: reflect.ValueOf(val).Elem(),
		typ:   typ.Elem(),
	}, nil
}

func (r *ReflectAccessor) Field(fieldName string) (any, error) {
	if _, ok := r.typ.FieldByName(fieldName); !ok {
		return 0, errors.New("非法字段")
	}
	return r.value.FieldByName(fieldName).Interface().(int), nil
}

func (r *ReflectAccessor) SetField(fieldName string, val int) error {
	if _, ok := r.typ.FieldByName(fieldName); !ok {
		return errors.New("非法字段")
	}
	fdVal := r.value.FieldByName(fieldName)
	if !fdVal.CanSet() {
		return errors.New("无法设置新值的字段")
	}
	fdVal.Set(reflect.ValueOf(val))
	return nil
}
