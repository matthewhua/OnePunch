//go:build answer

package service

import (
	"context"
	"log"
	"net/http"
	"os"
	"os/signal"
	"time"
)

// Option 典型的 Option 设计模式
type Option func(*App)

// ShutdownCallback 采用 context.Context 来控制超时，而不是用 time.After 是因为
// - 超时本质上是使用这个回调的人控制的
// - 我们还希望用户知道，他的回调必须要在一定时间内处理完毕，而且他必须显式处理超时错误
type ShutdownCallback func(ctx context.Context)

func WithShutdownCallbacks(cbs ...ShutdownCallback) Option {
	return func(app *App) {
		app.cbs = cbs
	}
}

type App struct {
	servers []*Server

	// 优雅退出整个超时时间，默认30秒
	shutdownTimeout time.Duration

	// 优雅退出时候等待处理已有请求时间，默认10秒钟
	waitTime time.Duration
	// 自定义回调超时时间，默认三秒钟
	cbTimeout time.Duration

	cbs []ShutdownCallback
}

func NewApp(servers []*Server, options ...Option) *App {
	app := &App{
		servers:         servers,
		shutdownTimeout: 30 * time.Second,
		waitTime:        10 * time.Second,
		cbTimeout:       3 * time.Second,
	}

	for _, opt := range opts {
		opt(app)
	}
	return app
}

// StartAndServe 你主要要实现这个方法
func (app *App) StartAndServe() {
	for _, s := range app.servers {
		srv := s
		go func() {
			if err := srv.Start(); err != nil {
				if err == http.ErrServerClosed {
					log.Printf("服务器%s已关闭", srv.name)
				} else {
					log.Printf("服务器%s异常退出", srv.name)
				}
			}
		}()
	}

	// 从这里开始开始启动监听系统信号
	// ch := make(...) 首先创建一个接收系统信号的 channel ch
	// 定义要监听的目标信号 signals []os.Signal
	// 调用 signal
	ch := make(chan os.Signal, 2)
	signal.Notify(ch, signals...)
	<-ch
	println("hello")
	go func() {
		select {
		case <-ch:
			log.Println("强制退出")
			os.Exit(1)
		case <-time.After(app.shutdownTimeout):
			log.Println("超时强制退出")
			os.Exit(1)
		}
	}()
	app.shutdown()
}

// shutdown 你要设计这里面的执行步骤。
func (app *App) shutdown() {
	log.Println("开始关闭应用，停止接收新请求")
	for _, s := range app.servers {
		// 思考：这里为什么我可以不用并发控制，即不用锁，也不用原子操作
		s.re
	}

	log.Println("等待正在执行请求完结")
	// 在这里等待一段时间

	log.Println("开始关闭服务器")
	// 并发关闭服务器，同时要注意协调所有的 server 都关闭之后才能步入下一个阶段

	log.Println("开始执行自定义回调")
	// 并发执行回调，要注意协调所有的回调都执行完才会步入下一个阶段

	// 释放资源
	log.Println("开始释放资源")
	app.close()
}
