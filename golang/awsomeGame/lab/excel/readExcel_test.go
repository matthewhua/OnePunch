package main

import (
	"errors"
	"fmt"
	"github.com/tealeg/xlsx"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
	"time"
)

func TestReadExcelFiles(t *testing.T) {
	files, err := filepath.Glob("D:/company/克隆计划/策划/数据表/数据表_开发/国服_版本1.0/*.xlsx")
	if err != nil {
		t.Fatalf("Failed to get files: %v", err)
	}

	for _, file := range files {
		xlFile, err := xlsx.OpenFile(file)
		if err != nil {
			t.Errorf("Failed to open file %s: %v", file, err)
			continue
		}

		// Iterate over the sheets and print their content.
		for _, sheet := range xlFile.Sheets {
			for _, row := range sheet.Rows {
				for _, cell := range row.Cells {
					text := cell.String()
					t.Logf("%s\t", text)
				}
				t.Log()
			}
		}
	}
}

type SheetData struct {
	Name   string
	Fields []string
	Data   []map[string]interface{} // 修改为interface{}类型，以便处理各种类型的数据
}

func TestReadExcelFiles1(t *testing.T) {
	files, err := filepath.Glob("D:/company/克隆计划/策划/数据表/数据表_开发/国服_版本1.0/*.xlsx")
	if err != nil {
		t.Fatal(err) // 使用t.Fatal来处理错误
		return
	}

	var allSheets []SheetData

	for _, file := range files {
		xlFile, err := xlsx.OpenFile(file)
		if err != nil {
			t.Error(err) // 使用t.Error来处理错误
			continue
		}

		for _, xlSheet := range xlFile.Sheets {
			sheetData := SheetData{
				Name: strings.Title(xlSheet.Name),
			}

			for rowIndex, row := range xlSheet.Rows {
				rowData := make(map[string]interface{}) // 修改为interface{}类型，以便处理各种类型的数据

				for cellIndex, cell := range row.Cells {
					var value interface{}
					switch cell.Type() {
					case xlsx.CellTypeString:
						value = cell.String()
					case xlsx.CellTypeNumeric:
						value, _ = cell.Float()
					case xlsx.CellTypeBool:
						value = cell.Bool()
					case xlsx.CellTypeDate:
						value, _ = cell.GetTime(false)
					default:
						value = cell.Value
					}

					if rowIndex == 0 {
						sheetData.Fields = append(sheetData.Fields, value.(string))
					} else {
						if cellIndex < len(sheetData.Fields) {
							fieldName := sheetData.Fields[cellIndex]
							rowData[fieldName] = value
						} else {
							rowData[fmt.Sprintf("Field%d", cellIndex+1)] = ""
						}
					}
				}

				if rowIndex != 0 {
					sheetData.Data = append(sheetData.Data, rowData)
				}
			}

			allSheets = append(allSheets, sheetData)
		}
	}

	for _, sheet := range allSheets {
		var fields []reflect.StructField
		for _, fieldName := range sheet.Fields {
			fields = append(fields, reflect.StructField{
				Name: strings.Title(fieldName),
				Type: reflect.TypeOf(""),
				Tag:  reflect.StructTag(fmt.Sprintf(`json:"%s"`, fieldName)),
			})
		}

		dynamicType := reflect.StructOf(fields)

		for _, rowData := range sheet.Data {
			dynamicValue := reflect.New(dynamicType).Elem()

			for fieldName, fieldValue := range rowData {
				field := dynamicValue.FieldByName(strings.Title(fieldName))
				if field.IsValid() {
					switch v := fieldValue.(type) {
					case string:
						field.SetString(v)
					case float64:
						if field.Type().Kind() == reflect.Float64 {
							field.SetFloat(v)
						} else {
							field.SetInt(int64(v))
						}
					case bool:
						if field.Type().Kind() == reflect.Bool {
							field.SetBool(v)
						}
					case time.Time:
						if field.Type().Kind() == reflect.Struct {
							field.Set(reflect.ValueOf(v))
						}
					default:
						t.Error(errors.New("unsupported data type"))
					}
				}
			}

		}
	}
	err = WriteToFile("output.json", allSheets)
	if err != nil {
		t.Error(fmt.Errorf("failed to write data to file: %w", err))
	}
}
