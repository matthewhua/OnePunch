package demo3

import "log"

type router struct {
	// trees 是按照 HTTP 方法来组织的
	// 如 GET => *node
	trees map[string]*node
}

func newRouter() router {
	return router{
		trees: map[string]*node{},
	}
}

// node 代表路由树的节点
// 路由树的匹配顺序是：
// 1. 静态完全匹配
// 2. 路径参数匹配：形式 :param_name
// 3. 通配符匹配：*
// 这是不回溯匹配
type node struct {
	path  string
	route string

	// children 子节点
	// 子节点的 path => node
	children map[string]*node
	// handler 命中路由之后执行的逻辑
	handler HandleFunc

	// 通配符 * 表达的节点, 任意匹配
	starChild *node

	paramChild *node
}

// child 返回子节点
// 第一个返回值 *node 是命中的节点
// 第二个返回值 bool 代表是否是命中参数路由
// 第三个返回值 bool 代表是否命中
func (n *node) childOf(path string) (*node, bool, bool) {
	if n.children == nil {
		if n.paramChild != nil {
			return n.paramChild, true, true
		}
		return n.starChild, false, n.starChild != nil
	}
	res, ok := n.children[path]
	if !ok {
		if n.paramChild != nil {
			return n.paramChild, true, true
		}
		return n.starChild, false, n.starChild != nil
	}
	return res, false, ok
}

// childOrCreate 查找子节点，
// 首先会判断 path 是不是通配符路径
// 其次判断 path 是不是参数路径，即以 : 开头的路径
// 最后会从 children 里面查找，
// 如果没有找到，那么会创建一个新的节点，并且保存在 node 里面
func (n *node) childOrCreate(path string) *node {
	if path == "*" {
		if n.paramChild != nil {
			if Debug {
				log.Println("xxxx Debugging")
			}
			if n.starChild == nil {
				n.starChild = &node{path: path}
			}
			return n.starChild
		}
	}
}

type matchInfo struct {
	n          *node
	pathParams map[string]string
}
