package main

import (
	"fmt"
	"github.com/tealeg/xlsx"
	"log"
	"path/filepath"
)

func main() {
	// 打开 Excel 文件
	baseDir := "D:/company/克隆计划/策划/数据表/数据表_开发/国服_版本1.0/"
	fileName := "功能_我要变强.xlsx"
	filePath := filepath.Join(baseDir, fileName)

	// 打开 Excel 文件
	file, err := xlsx.OpenFile(filePath)
	if err != nil {
		log.Fatal(err)
	}
	// 遍历每个工作表
	for _, sheet := range file.Sheets {
		fmt.Printf("Sheet Name: %s\n", sheet.Name)

		// 遍历每行数据
		for _, row := range sheet.Rows {
			// 遍历每个单元格
			for _, cell := range row.Cells {
				value, err := cell.FormattedValue()
				if err != nil {
					log.Fatal(err)
				}
				fmt.Printf("%s\t", value)
			}
			fmt.Println()
		}

		fmt.Println()
	}
}
