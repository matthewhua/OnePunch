package main

import (
	"encoding/json"
	"fmt"
	"github.com/tealeg/xlsx"
	"log"
	"os"
	"path/filepath"
)

func WriteToFile(filename string, data interface{}) error {
	file, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to create file: %w", err)
	}
	defer file.Close()

	jsonData, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal data: %w", err)
	}

	_, err = file.Write(jsonData)
	if err != nil {
		return fmt.Errorf("failed to write data to file: %w", err)
	}

	return nil

}

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
