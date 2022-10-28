// 柯里化演示

//函数的柯里化
function checkAge(min) {
    return function (age) {
        return age >= min
    }
}

// ES6 箭头函数
let checkTime = min => (time => time >= min)

let checkAge18 = checkAge(18)
let minute30 = checkTime(30)

console.log(checkAge18(20))
console.log(checkAge18(24))
console.log(checkAge18(22))
console.log(minute30(99))