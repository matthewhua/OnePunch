package net

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestClient_Send(t *testing.T) {

	testCases := []struct {
		req  string
		resp string
	}{
		{
			req:  "hello",
			resp: "hello, from response",
		},
		{
			req:  "aaa bbb ccc \n",
			resp: "aaa bbb ccc \n, from response",
		},

		{
			req:  "i'm Matthew",
			resp: "i'm Matthew",
		},
	}

	c := &Client{
		addr: "localhost:8080",
	}
	for _, tc := range testCases {
		t.Run(tc.req, func(t *testing.T) {
			resp, err := c.Send(tc.req)
			assert.Nil(t, err)
			assert.Equal(t, tc.resp, resp)
		})
	}
}
