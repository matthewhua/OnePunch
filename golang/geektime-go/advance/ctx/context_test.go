package ctx

import (
	"context"
	"fmt"
	"golang.org/x/sync/errgroup"
	"sync/atomic"
	"testing"
	"time"
)

func TestErrgroup(t *testing.T) {
	eg, ctx := errgroup.WithContext(context.Background())
	var result int64 = 0
	for i := 0; i < 10; i++ {
		delta := i
		eg.Go(func() error {
			atomic.AddInt64(&result, int64(delta))
			return nil
		})
	}
	if err := eg.Wait(); err != nil {
		t.Fatal(err)
	}
	ctx.Err()
	fmt.Println(result)
}

func TestBusinessTimeOut(t *testing.T) {
	ctx := context.Background()
	timeout, cancel := context.WithTimeout(ctx, time.Second)
	defer cancel()
	end := make(chan struct{}, 1)
	go func() {
		MyBusiness()
		end <- struct{}{}
	}()
	ch := timeout.Done()
	select {
	case <-ch:
		fmt.Println("timeout")
	case <-end:
		fmt.Println("business end")
	}
}

func MyBusiness() {
	time.Sleep(500 * time.Millisecond)
	fmt.Println("hello world")
}

func TestParentValueCtx(t *testing.T) {
	ctx := context.Background()
	childCtx := context.WithValue(ctx, "map", map[string]string{})
	ccChild := context.WithValue(childCtx, "key1", "value1")
	m := ccChild.Value("map").(map[string]string)
	m["key1"] = "val1"
	val := childCtx.Value("key1")
	fmt.Println(val)
	val = childCtx.Value("map")
	fmt.Println(val)
}

func TestContext(t *testing.T) {
	ctx := context.Background()
	valCtx := context.WithValue(ctx, "abc", 123)
	value := valCtx.Value("abc")
	fmt.Println(value)
}

func TestContext_timeout(t *testing.T) {
	bg := context.Background()
	timeoutCtx, cancel1 := context.WithTimeout(bg, time.Second)
	subCtx, cancel2 := context.WithTimeout(timeoutCtx, 3*time.Second)
	go func() {
		// 一秒钟之后就会过期，然后输出 timeout
		<-subCtx.Done()
		fmt.Printf("timout")
	}()

	time.Sleep(2 * time.Second)
	cancel2()
	cancel1()
}

func TestTimeoutExample(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	bsChan := make(chan struct{})
	go func() {
		slowBusiness()
		bsChan <- struct{}{}
	}()
	select {
	case <-ctx.Done():
		fmt.Println("timeout")
	case <-bsChan:
		fmt.Println("business end")
	}
}

func slowBusiness() {
	time.Sleep(2 * time.Second)
}

func TestTimeoutTimeAfter(t *testing.T) {

	bsChan := make(chan struct{})
	go func() {
		slowBusiness()
		bsChan <- struct{}{}
	}()
	timer := time.AfterFunc(time.Second, func() {
		fmt.Println("timeout")
	})
	<-bsChan
	fmt.Println("business end")
	timer.Stop()
}
