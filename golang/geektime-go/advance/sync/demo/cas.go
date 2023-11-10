package demo

type Lock struct {
	state int
}

func (l *Lock) CAS(oldValue int, newValue int) {
	if l.state == oldValue {
		l.state = newValue
	}
}
