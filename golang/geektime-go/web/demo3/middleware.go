package demo3

type Middleware func(next HandleFunc) HandleFunc
