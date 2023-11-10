package types

type User struct {
	Name string
	// 因为同属一个包，所以 age 还可以被测试访问到
	// 如果是不同包，就访问不到了
	age int
}

func (u User) GetName() string {
	return u.Name
}
