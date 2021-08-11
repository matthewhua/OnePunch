package ch2

import "testing"

const (
	Monday = 1 + iota
	Tuesday
	Wednesday
)

const (
	Readable = 1 << iota
	Writable
	Executable
)

func TestConstanceTry(t *testing.T)  {
	t.Log(Monday, Tuesday)
}

func TestConstantTry1(t *testing.T) {
	a := 1
	t.Log(a&Readable == Readable, a&Writable == Writable, a&Executable == Executable)
}