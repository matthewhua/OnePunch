package config

import (
	"errors"
	"github.com/Unknwon/goconfig"
	"log"
	"os"
	"path/filepath"
)

const EnvConfigFilePathKey = "ZINX_CONFIG_FILE_PATH"
const configFile = "/conf/conf.ini"

var File *goconfig.ConfigFile

var env = new(zEnv)

type zEnv struct {
	configFilePath string
}

//加载此文件的时候 会先走初始化方法

func init() {
	//拿到当前的程序的目录
	configFilePath := os.Getenv(EnvConfigFilePathKey)
	if configFilePath == "" {
		pwd, err := os.Getwd()
		if err != nil {
			panic(err)
		}
		configFilePath = filepath.Join(pwd, configFile)
	}
	var err error
	configFilePath, err = filepath.Abs(configFilePath)
	if err != nil {
		panic(err)
	}
	env.configFilePath = configFilePath
	if !fileExist(configFilePath) {
		panic(errors.New("配置文件不存在"))
	}
	//参数  mssgserver.exe  D:/xxx

	len := len(os.Args)
	if len > 1 {
		dir := os.Args[1]
		if dir != "" {
			configFilePath = dir + configFile
		}
	}

	// 文件系统的读取
	File, err = goconfig.LoadConfigFile(configFile)
	if err != nil {
		log.Fatal("读取配置文件出错:", err)
	}
}

func fileExist(fileName string) bool {
	_, err := os.Stat(fileName)
	return err == nil || os.IsExist(err)
}

func GetConfigFilePath() string {
	return env.configFilePath
}
