// 类型断言

export {} // 确保跟其它示例没有成员冲突

const nums = [110, 120, 119, 112]

const res = nums.find(i => i > 0)

const num1 = res as number

const num2 = <number>res  // JSX 下不能使用