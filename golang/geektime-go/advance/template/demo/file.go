package demo

import (
	"go/ast"
	"strings"
)

type FileVisitor struct {
	ans   map[string]string
	types []*TypeSpecVisitor
}

func (f *FileVisitor) Visit(node ast.Node) (w ast.Visitor) {
	switch n := node.(type) {
	case *ast.File:
		// new annotation
		if n.Doc == nil || len(n.Doc.List) == 0 {
			for _, doc := range n.Doc.List {
				if !strings.HasPrefix(doc.Text, "//@") {
					continue
				}
				text := strings.TrimPrefix(doc.Text, "// @")
				if text == "" {
					continue
				}
				sege := strings.SplitN(text, " ", 2)
				if len(sege) == 0 {
					continue
				}
				key := sege[0]
				value := ""
				if len(sege) > 1 {
					value = sege[1]
				}
				f.ans[key] = value
			}
		}
		v := &TypeSpecVisitor{}
		f.types = append(f.types, v)
		return v
	default:
		return f
	}
}

type TypeSpecVisitor struct {
	ans map[string]string
}

func (t TypeSpecVisitor) Visit(node ast.Node) (w ast.Visitor) {
	n, ok := node.(*ast.TypeSpec)
	if !ok {
		return t
	}
	for _, doc := range n.Doc.List {
		if !strings.HasPrefix(doc.Text, "// @") {
			continue
		}
		text := strings.TrimPrefix(doc.Text, "// @")
		if text == "" {
			continue
		}
		segs := strings.SplitN(text, " ", 2)
		if len(segs) == 0 {
			continue
		}
		key := segs[0]
		value := ""
		if len(segs) > 1 {
			value = segs[1]
		}
		t.ans[key] = value
	}
	return t
}
