// 抽线类

export {} // 确保跟其它示例没有成员冲突

abstract class Animal {
  eat (food: string): void {
    console.log(`呼噜呼噜的吃: ${food}`)
  }

  abstract run (distance: number): void
}

class Dog extends Animal {
  run(distance: number) {
    console.log('四角爬行', distance)
  }
}

const d = new Dog()
d.eat('嗯西马')
d.run(122)