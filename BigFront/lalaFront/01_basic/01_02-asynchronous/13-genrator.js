// 生成器函数

function * foo () {
    console.log('start')

    try {
        let res = yield 'foo';
        console.log(res)
    } catch (e) {
        console.log(e)
    }
}

const generator = foo()

const result = generator.next()
console.log(result)


generator.throw(new Error('Generator error'))