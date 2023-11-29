package demo3

import (
	"mime/multipart"
	"path/filepath"
)

type FileUploader struct {
	FileField string
	// 比如说 DST 是一个目录
	Dst string
	// DstPathFunc 用于计算目标路径
	DstPathFunc func(fh *multipart.FileHeader) string
}

func (f *FileUploader) Handle() HandleFunc {
	return func(ctx *Context) {

	}
}

type FileDownloader struct {
	// 设计各种参数
	Dir string
}

func (f *FileDownloader) Handle() HandleFunc {
	// 你可以在这里
	return func(ctx *Context) {
		// file 也可以是多段的呀 /a/b/c.txt
		file, err := ctx.QueryValue("file")
		if err != nil {
			ctx.RespStatusCode = 500
			ctx.RespData = []byte("文件找不到")
			return
		}
		// file 可能是一些乱七八糟的东西
		// file =///////abc.txt
		path := filepath.Join(f.Dir, filepath.Clean(file))
		// 从完整路径里面拿到文件名
		fn := filepath.Join(f.Dir, filepath.Clean(file))

		// 从完整路径里面拿到文件名
		fn := filepath.Base(path)

	}
}
