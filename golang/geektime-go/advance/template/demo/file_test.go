package demo

import (
	"github.com/stretchr/testify/assert"
	"go/ast"
	"go/parser"
	"go/token"
	"testing"
)

func TestFileVisitor(t *testing.T) {
	fset := token.NewFileSet()
	f, err := parser.ParseFile(fset, "src.go",
		`
// annotation go through the source code and extra the annotation
// @author Matthew Hua
// @date 2023/11/14
// @
package annotation

type (
	// Interface is a test interface
	// @author Matthew Hua
	/* @multiple first line
	   second line
	*/
	// @date 2023/11/14
	Interface interface {
		// MyFunc is a test func
		// @parameter arg1 int
		// @parameter arg2 int32
		// @return string
		MyFunc(arg1 int, arg2 int32) string

		// second is a test func
		// @return string
		second() string
	}
)
`, parser.ParseComments)
	if err != nil {
		t.Fatal(err)
	}
	fv := &FileVisitor{
		ans:   map[string]string{},
		types: []*TypeSpecVisitor{},
	}
	ast.Walk(fv, f)
	res := map[string]string{
		"date":   "2023/11/14",
		"author": "Matthew Hua",
	}
	assert.Equal(t, res, fv.ans)
}
