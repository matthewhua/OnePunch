package gen_trans

import (
	"bufio"
	"fmt"
	"io/ioutil"
	"os"
	"regexp"
	"strings"
	"text/template"
)

type MsgType struct {
	Name    string
	MsgType int
	Comment string
}

func Trans() {
	filePath := "MsgType.kt" // 替换为实际的MsgType文件路径

	// 读取MsgType文件
	file, err := os.Open(filePath)
	if err != nil {
		fmt.Printf("Failed to open MsgType file: %v", err)
		os.Exit(1)
	}
	defer file.Close()

	// 解析MsgType文件
	messageList := parseMsgType(file)

	protoTemplate := `
syntax = "proto3";

package messageId; // 替换为您的包名

import "google/protobuf/any.proto"; // 根据需要导入其他依赖的.proto文件

enum MsgType {
{{range .}}
	{{.Name}} = {{.MsgType}};{{if .Comment}} // {{.Comment}}{{end}}
{{end}}
}

// 导入其他依赖的消息类型
{{range .}}
import "{{.Name}}.proto"; // 导入{{.Name}}消息类型的定义{{end}}
// 导入其他消息类型...

// 定义其他消息类型...
`

	tmpl, err := template.New("proto").Parse(protoTemplate)
	if err != nil {
		fmt.Printf("Failed to parse proto template: %v", err)
		os.Exit(1)
	}

	file, err = os.Create("messages.proto")
	if err != nil {
		fmt.Printf("Failed to create proto file: %v", err)
		os.Exit(1)
	}
	defer file.Close()

	err = tmpl.Execute(file, messageList)
	if err != nil {
		fmt.Printf("Failed to generate proto file: %v", err)
		os.Exit(1)
	}

	fmt.Println("Generated pb file successfully!")
}

func convertMultilineToSingleline(filePath string) {
	// 读取源代码文件
	sourceCode, err := ioutil.ReadFile(filePath)
	if err != nil {
		fmt.Println("无法读取文件：", err)
		return
	}

	// 匹配多行消息定义并转换为单行形式
	pattern := `(\w+)\([\s\S]*?\),\s*//\s*([\s\S]*?)\n`
	re := regexp.MustCompile(pattern)
	result := re.ReplaceAllStringFunc(string(sourceCode), func(match string) string {
		// 提取消息号和注释
		submatch := re.FindStringSubmatch(match)
		messageNum := submatch[1]
		comment := submatch[2]

		// 构建新的消息定义
		newLine := fmt.Sprintf("%s // %s", messageNum, comment)
		return newLine
	})

	// 写入修改后的代码文件
	modifiedFilePath := "ModifiedCode.kt"
	err = ioutil.WriteFile(modifiedFilePath, []byte(result), 0644)
	if err != nil {
		fmt.Println("无法写入文件：", err)
		return
	}

	fmt.Println("已生成修改后的代码文件：", modifiedFilePath)
}

// 解析MsgType文件
func parseMsgType(file *os.File) []MsgType {
	scanner := bufio.NewScanner(file)
	messageList := make([]MsgType, 0)

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if strings.HasPrefix(line, "enum class MsgType") {
			messageList = parseEnum(scanner)
			break
		}
	}

	return messageList
}

// 解析enum MsgType部分
func parseEnum(scanner *bufio.Scanner) []MsgType {
	messageList := make([]MsgType, 0)

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "}" {
			break
		}
		if strings.HasPrefix(line, "//") {
			continue
		}
		if strings.Contains(line, "_") {
			parts := strings.Split(line, "_")
			name := strings.TrimSpace(parts[0])
			msgType := strings.TrimSpace(parts[len(parts)-1])
			comment := ""
			if strings.Contains(line, "//") {
				commentParts := strings.Split(line, "//")
				comment = strings.TrimSpace(commentParts[1])
			}
			message := MsgType{
				Name:    name,
				MsgType: parseInt(msgType),
				Comment: comment,
			}
			messageList = append(messageList, message)
		}
		if strings.HasPrefix(line, "), ") && strings.Contains(line, "//") {
			last := messageList[len(messageList)-1]
			if last.Comment == "" {
				commentParts := strings.Split(line, "//")
				messageList[len(messageList)-1].Comment = strings.TrimSpace(commentParts[1])
			}
		}
	}

	return messageList
}

// 解析整数
func parseInt(s string) int {
	var result int
	_, err := fmt.Sscanf(s, "%d", &result)
	if err != nil {
		fmt.Printf("Failed to parse integer: %v %s", err, s)
		os.Exit(1)
	}
	return result
}
