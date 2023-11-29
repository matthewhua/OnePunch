package accesslog

import (
	"encoding/json"
	web "geektime-go/web/demo3"
	"io"
)

type MiddlewareBuilder struct {
	logFunc func(accesslog []byte)
}

func (m MiddlewareBuilder) Build() web.Middleware {
	return func(next web.HandleFunc) web.HandleFunc {
		return func(ctx *web.Context) {
			body, err := io.ReadAll(ctx.Req.Body)
			log := accessLog{
				Method: ctx.Req.Method,
				Body:   string(body),
			}
			bs, err := json.Marshal(log)
			if err == nil {
				m.logFunc(bs)
			}

			// before route, before exec
			next(ctx)
			// 这里就是 after execute, after route
			// m.logFunc(ctx.Resp)
		}
	}
}

type accessLog struct {
	Method string
	Body   string
}
