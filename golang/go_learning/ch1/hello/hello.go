package main

import (
	"fmt"
	"os"
)

func main()  {
	if len(os.Args) > 1 {
		fmt.Println("Hello Matthew", os.Args[1])
	}
}