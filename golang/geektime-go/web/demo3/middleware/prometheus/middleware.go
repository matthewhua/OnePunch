package prometheus

import (
	"fmt"
	web "geektime-go/web/demo3"
	"github.com/prometheus/client_golang/prometheus"
	_ "github.com/prometheus/client_golang/prometheus"
	"time"
)

type MiddlewareBuilder struct {
	Name        string
	Subsystem   string
	ConstLabels map[string]string
	Help        string
}

func (b *MiddlewareBuilder) Build() web.Middleware {
	summaryVec := prometheus.NewSummaryVec(prometheus.SummaryOpts{
		Name:        b.Name,
		Subsystem:   b.Subsystem,
		ConstLabels: b.ConstLabels,
		Help:        b.Help,
	}, []string{"pattern", "method", "status"})

	return func(next web.HandleFunc) web.HandleFunc {
		return func(ctx *web.Context) {
			startTime := time.Now()
			defer func() {
				endTime := time.Now()
				duration := endTime.Sub(startTime).Milliseconds()
				summary := summaryVec.WithLabelValues(ctx.MatchedRoute,
					ctx.Req.Method, fmt.Sprintf("%d", ctx.RespStatusCode))
				summary.Observe(float64(duration))
			}()
			next(ctx)
		}
	}
}
