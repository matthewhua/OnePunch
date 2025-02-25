// 高阶函数-函数作为返回值

// once 只执行一次
function once (fn) {
    let done = false
    return function () {
        if (!done) {
            done = true
            return fn.apply(this, arguments)
        }
    }
}

let pay = once(function (money) {
    console.log(`支付: ${money} RMB`)
})

pay(3)
pay(4)
pay(1)