// 函数类型

export {} // 确保跟其它示例没有成员冲突

function func1(a: number, b: number = 10, ...rest: number[]): string {
  return 'func1'
}

console.log(func1(100, 200))

console.log(func1(100))

console.log(func1(100, 200, 300))

const func2: (a: number, b: number) => string = function (a: number, b: number): string {
  return 'func2'
}

func2(1, 2)