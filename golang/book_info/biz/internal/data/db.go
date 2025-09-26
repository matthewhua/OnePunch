package data

import (
  "fmt"
  "github.com/spf13/viper"
  "gorm.io/driver/sqlite"
  "gorm.io/gorm"
  "os"
  "path/filepath"
)

var DB *gorm.DB

func InitDB(dbPath string) error {
	// 如果数据库文件不存在，则创建它
	dpPath := viper.GetString("sql")
	dir := filepath.Dir(dpPath)
	if _, err := os.Stat(dpPath); os.IsNotExist(err) {
}
