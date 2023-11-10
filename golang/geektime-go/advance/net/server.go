package net

import (
	"encoding/binary"
	"fmt"
	"io"
	"net"
)

func Serve(addr string) error {
	listener, err := net.Listen("tcp", addr)
	if err != nil {
		return err
	}
	for {
		conn, err := listener.Accept()
		if err != nil {
			return err
		}
		go func() {
			handleCoon(conn)
		}()
	}
}

func handleCoon(conn net.Conn) {
	for {
		// 读数据
		bytes := make([]byte, 8)
		_, err := conn.Read(bytes)
		if err == io.EOF || err == net.ErrClosed || err == io.ErrUnexpectedEOF {
			// 一般关闭的错误比较懒得管
			// 也可以把关闭错误输出到日志
			_ = conn.Close()
			return
		}
		if err != nil {
			continue
		}
		res := handleMsg(bytes)
		_, err = conn.Write(res)
		if err == io.EOF || err == net.ErrClosed ||
			err == io.ErrUnexpectedEOF {
			_ = conn.Close()
			return
		}
	}
}

func handleMsg(bs []byte) []byte {
	return []byte("world")
}

type Server struct {
	addr string
}

func (s *Server) StartAndServe() error {
	listener, err := net.Listen("tcp", s.addr)
	if err != nil {
		return err
	}
	for {
		conn, err := listener.Accept()
		if err != nil {
			return err
		}
		go func() {
			// 直接在这里处理
			er := s.handleConn(conn)
			if er != nil {
				_ = conn.Close()
				fmt.Printf("close conn: %v", er)
			}
		}()
	}
}

func (s *Server) handleConn(conn net.Conn) error {
	for {
		// 读数据
		bytes := make([]byte, 8)
		_, err := conn.Read(bytes)
		if err != nil {
			return err
		}
		reqBs := make([]byte, binary.BigEndian.Uint64(bytes))
		_, err = conn.Read(reqBs)
		if err != nil {
			return err
		}

		res := string(reqBs) + ", from response"
		// 总长度
		bytes = make([]byte, lenBytes, len(res)+lenBytes)
		// 写入消息长度
		binary.BigEndian.PutUint64(bytes, uint64(len(res)))
		bytes = append(bytes, res...)
		_, err = conn.Write(bytes)
		if err != nil {
			return err
		}
	}
}
